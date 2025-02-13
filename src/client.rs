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
        let next_write = (header.write_index + 1) % CAPACITY;
        
        // Check if queue is full
        if next_write == header.read_index {
            return Err("Queue is full".into());
        }

        // Calculate where to write the new request
        let requests_ptr = (ptr as *mut u8).add(size_of::<Header>());
        let request_slot = requests_ptr.add(header.write_index * size_of::<Request>());
        
        // Write the request
        std::ptr::copy_nonoverlapping(
            &request as *const Request as *const u8,
            request_slot,
            size_of::<Request>()
        );

        println!("Client: Inserted request at position {} - {}", header.write_index, request);
        
        // Update write index
        header.write_index = next_write;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let ptr = setup_shared_memory_client()?;
    let request1 = Request::new(Operation::INSERT, "test_key", "test_value");
    let request2 = Request::new(Operation::INSERT, "test_key2", "test_value2");
    let request3 = Request::new(Operation::INSERT, "test_key3", "test_value3");
    add_request(ptr, request1)?;
    add_request(ptr, request2)?;
    add_request(ptr, request3)?;

    let request1 = Request::new(Operation::GET, "test_key", "Dummy value");
    add_request(ptr, request1)?;

    Ok(())
}
