mod lib;
use lib::{Operation, Request, Header, SHARED_MEMORY_SIZE, CAPACITY};
use nix::sys::{mman, mman::ProtFlags, mman::MapFlags};
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use std::error::Error;
use std::mem::size_of;
use std::num::NonZero;

fn setup_shared_memory_client() -> Result<*mut u8, Box<dyn Error>> {
    let shm_fd = mman::shm_open(
        "RequestQueue",
        OFlag::O_RDWR,
        Mode::empty(),
    )?;

    let ptr = unsafe { 
        mman::mmap(
            None, 
            NonZero::new(SHARED_MEMORY_SIZE).unwrap(), 
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, 
            MapFlags::MAP_SHARED, 
            shm_fd, 
            0)? 
    };

    Ok(ptr.as_ptr() as *mut u8)
}

fn add_request(ptr: *mut u8, request: Request) -> Result<(), Box<dyn Error>> {
    unsafe {
        let header_ptr = ptr as *mut Header;
        let header = &mut *header_ptr;

        loop {
            // Try to acquire write lock
            let write_result = header.write_index.try_write();
            if write_result.is_err() {
                println!("Client: Waiting for write lock on write index");
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            let mut write_guard = write_result.unwrap();

            // Try to acquire read lock
            let read_result = header.read_index.try_read();
            if read_result.is_err() {
                println!("Client: Waiting for read lock on read index");
                std::thread::sleep(std::time::Duration::from_millis(100));
                // write_guard is dropped here automatically
                continue;
            }
            let read_guard = read_result.unwrap();

            let next_write = (*write_guard + 1) % CAPACITY;
            
            if next_write == *read_guard {
                return Err("Client: Queue is full".into());
            }

            // Calculate where to write the new request
            let requests_ptr = (ptr as *mut u8).add(size_of::<Header>());
            let request_slot = requests_ptr.add(*write_guard * size_of::<Request>());
            
            // Write the request
            std::ptr::copy_nonoverlapping(
                &request as *const Request as *const u8,
                request_slot,
                size_of::<Request>()
            );

            println!("Client: Inserted request at position {} - {}", *write_guard, request);
            
            *write_guard = next_write;
            return Ok(());
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ptr = setup_shared_memory_client()?;
    let request1 = Request::new(Operation::INSERT, "test_key", "test_value");
    let request2 = Request::new(Operation::INSERT, "test_key2", "test_value2");
    let request3 = Request::new(Operation::INSERT, "test_key3", "test_value3");

    println!("Client: Adding requests");
    add_request(ptr, request1)?;
    add_request(ptr, request2)?;
    add_request(ptr, request3)?;

    let request1 = Request::new(Operation::GET, "test_key", "Dummy value");
    add_request(ptr, request1)?;

    Ok(())
}
