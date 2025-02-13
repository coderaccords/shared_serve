use shared_serve::{Operation, Request, Header, SHARED_MEMORY_SIZE, CAPACITY};
use nix::sys::{mman, mman::ProtFlags, mman::MapFlags};
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use std::error::Error;
use std::mem::size_of;
use std::num::NonZero;
use std::io::{self, Write};

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
            let write_index_result = header.write_index.try_write();
            if write_index_result.is_err() {
                println!("Client: Waiting for write lock on write index");
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            let mut write_index_guard = write_index_result.unwrap();

            // Try to acquire read lock
            let read_index_result = header.read_index.try_read();
            if read_index_result.is_err() {
                println!("Client: Waiting for read lock on read index");
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            let read_index_guard = read_index_result.unwrap();
            let next_write = (*write_index_guard + 1) % CAPACITY;
            
            if next_write == *read_index_guard {
                return Err("Client: Queue is full".into());
            }

            // Calculate where to write the new request
            let requests_ptr = (ptr as *mut u8).add(size_of::<Header>());
            let request_slot = requests_ptr.add(*write_index_guard * size_of::<Request>());
            
            // Write the request
            std::ptr::copy_nonoverlapping(
                &request as *const Request as *const u8,
                request_slot,
                size_of::<Request>()
            );

            println!("Client: Inserted request at position {} - {}", next_write, request);
            
            *write_index_guard = next_write;

            return Ok(());
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ptr = setup_shared_memory_client()?;

    loop {
        println!("\nAvailable operations:");
        println!("1. INSERT");
        println!("2. GET");
        println!("3. DELETE");
        println!("4. Exit");
        
        print!("Enter operation number: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let operation = match input.trim() {
            "1" => Operation::INSERT,
            "2" => Operation::GET,
            "3" => Operation::DELETE,
            "4" => break,
            _ => {
                println!("Invalid operation! Please try again.");
                continue;
            }
        };
        
        print!("Enter key: ");
        io::stdout().flush()?;
        let mut key = String::new();
        io::stdin().read_line(&mut key)?;
        let key = key.trim();
        
        let value = if operation == Operation::INSERT {
            print!("Enter value: ");
            io::stdout().flush()?;
            let mut value = String::new();
            io::stdin().read_line(&mut value)?;
            value.trim().to_string()
        } else {
            "".to_string()
        };
        
        let request = Request::new(operation, key, &value);
        match add_request(ptr, request) {
            Ok(_) => println!("Request added successfully!"),
            Err(e) => println!("Failed to add request: {}", e),
        }
        print!("================================================");
    }   
    Ok(())
}
