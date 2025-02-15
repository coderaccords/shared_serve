use std::thread;
use std::time::Duration;
use std::path::Path;

mod common;

#[test]
fn test_graceful_shutdown() {
    println!("Starting server");
    let mut server = common::start_server();
    thread::sleep(Duration::from_secs(2));

    println!("Starting client");
    let mut client = common::start_client();
    thread::sleep(Duration::from_secs(1));

    println!("Sending shutdown signal (SIGINT)");
    common::stop_server_with_sigint(&server);
    server.wait().expect("Failed to wait for server to exit");
    thread::sleep(Duration::from_secs(2));

    println!("Checking if shared memory exists");
    let shm_path = "/dev/shm/RequestQueue";
    assert!(!Path::new(shm_path).exists(), "Shared memory was not cleaned up");

    println!("Killing client");
    client.kill().expect("Failed to kill client");
} 