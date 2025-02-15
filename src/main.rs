use shared_serve::{HashTable, Operation, Request, Header, SHARED_MEMORY_SIZE, CAPACITY};
use clap::Parser;
use nix::sys::{mman, mman::ProtFlags, mman::MapFlags};
use nix::fcntl:: OFlag;
use nix::unistd::ftruncate;
use nix::sys::stat::Mode;
use std::error::Error;
use nix::libc::off_t;
use std::num::NonZero;
use std::os::fd::AsFd;
use std::ptr;
use threadpool::ThreadPool;
use std::sync::{Arc, mpsc::channel};
use ctrlc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "10")]
    size: usize,
    #[arg(short, long, default_value = "4")]
    num_threads: usize,
}

pub fn setup_shared_memory_server() -> Result<*mut u8, Box<dyn Error>> {
    // Create the shared memory object
    let shm_fd = mman::shm_open(
        "RequestQueue", 
        OFlag::O_CREAT | OFlag::O_RDWR , 
        Mode::S_IRUSR | Mode::S_IWUSR)?;
    
    ftruncate(
        shm_fd.as_fd(), 
        SHARED_MEMORY_SIZE as off_t)?;
    
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


pub fn get_request(ptr: *mut u8) -> Result<Request, Box<dyn Error>> {
    unsafe {
        let header_ptr = ptr as *mut Header;
        let header = &mut *header_ptr;
        loop {
            
            // Try to acquire read lock
            let write_index_result = header.write_index.try_read();
            if write_index_result.is_err() {
                continue;
            }
            let write_guard = write_index_result.unwrap();
            let write_index = *write_guard;

            // Try to acquire write lock
            let read_index_result = header.read_index.try_write();
            if read_index_result.is_err() {
                continue;
            }
            let mut read_index_guard = read_index_result.unwrap();
            let read_index = *read_index_guard;
           
            if read_index == write_index {
                return Err("Server: Queue is empty".into());
            }

            // Calculate where to read the request from
            let requests_ptr = ptr.add(size_of::<Header>());
            let request_slot = requests_ptr.add(read_index * size_of::<Request>()) as *const Request;
            
            // Read the request
            let request = ptr::read(request_slot);
            
            // Update read index
            *read_index_guard = (read_index + 1) % CAPACITY;

            println!("Server: Received request at position {} - {}", *write_guard, request);
            return Ok(request);
        }
    }
}

pub fn process_request(request: Request, hash_table: Arc<HashTable>) -> Result<(), Box<dyn Error>> {
    println!("Processing request: {}", request);
    // Process the request based on operation type
    match request.operation {
        Operation::INSERT => {
            println!("Inserting key: {}", request.key_str());
            hash_table.insert(request.key_str(), request.value_str());
        },
        Operation::DELETE => {
            println!("Deleting key: {}", request.key_str());
            let result = hash_table.delete(request.key_str());
            if result {
                println!("Key deleted successfully");
            } else {    
                println!("Key not found: {}", request.key_str());
            }
        },
        Operation::GET => {
            println!("Getting key: {}", request.key_str());
            match hash_table.get(request.key_str()) {
                Some(value) => println!("Value: {}", value),
                None => println!("Key not found:    // let ptr_arc = Arc::new(Mutex::new(SafePtr(ptr)));
 {}", request.key_str()),
            }
        },
    }
    Ok(())
}

fn cleanup(ptr: *mut u8) {
    println!("Cleaning up...");
    unsafe {
        // Unmap the shared memory
        if let Err(e) = mman::munmap(
            std::ptr::NonNull::new(ptr as *mut _).unwrap(),
            SHARED_MEMORY_SIZE
        ) {
            eprintln!("Error unmapping shared memory: {}", e);
        }
    }
    // Unlink the shared memory object
    if let Err(e) = mman::shm_unlink("RequestQueue") {
        eprintln!("Error unlinking shared memory: {}", e);
    }
    println!("Cleanup complete. Exiting.");
}

fn main() -> Result<(), Box<dyn Error>> {
    
    std::panic::set_hook(Box::new(|_| {
    }));

    let args = Args::parse();
    let hash_table_size = args.size;
    let thread_count = args.num_threads;

    let hash_table = Arc::new(HashTable::new(hash_table_size));
    let ptr = setup_shared_memory_server().expect("Failed to set up shared memory");

    

    let threads = ThreadPool::new(thread_count);

    let (shutdown_tx, shutdown_rx) = channel();

    ctrlc::set_handler(move || shutdown_tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    println!("Server started with {} threads. Waiting for requests...", thread_count);
    
    loop {
        
        if shutdown_rx.try_recv().is_ok() {
            println!("Shutdown signal received.");
            threads.join();
            break;
        }
        
        match get_request(ptr) {
            Ok(request) => {
                let hash_table = hash_table.clone();
                threads.execute(move || {
                    if let Err(e) = process_request(request, hash_table) {
                        eprintln!("Error processing request: {}", e);
                    }
                });
            },
            Err(e) => {
                if e.to_string() == "Server: Queue is empty" {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                else {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }

    
    cleanup(ptr);

    Ok(())
}
