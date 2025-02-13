mod lib;
use lib::{HashTable as HT, Operation, Request, Header, SHARED_MEMORY_SIZE, CAPACITY};
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

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "10")]
    size: usize,
}

pub fn setup_shared_memory_server() -> Result<*mut u8, Box<dyn Error>> {
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

        // Check if queue is empty
        if header.read_index == header.write_index {
            return Err("Queue is empty".into());
        }

        // Calculate where to read the request from
        let requests_ptr = ptr.add(size_of::<Header>());
        let request_slot = requests_ptr.add(header.read_index * size_of::<Request>()) as *const Request;

        // Read the request
        let request = ptr::read(request_slot);

        println!("Server: Read request at position {} - {}", header.read_index, request);
        // Update read index
        header.read_index = (header.read_index + 1) % CAPACITY;


        Ok(request)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let hash_table_size = args.size;
    let mut hash_table = HT::new(hash_table_size);
    let ptr = setup_shared_memory_server()?;

    println!("Server started. Waiting for requests...");
    
    loop {
        match get_request(ptr) {
            Ok(request) => {
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
                            None => println!("Key not found: {}", request.key_str()),
                        }
                    },
                }
            },
            Err(e) => {
                if e.to_string() == "Queue is empty" {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                eprintln!("Error: {}", e);
            }
        }
    }
}
