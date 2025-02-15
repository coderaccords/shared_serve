use std::thread;
use std::time::Duration;
use std::path::Path;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
mod common;

#[test]
fn test_graceful_shutdown() {
    println!("Starting server");
    let server = common::start_server();
    thread::sleep(Duration::from_secs(2)); // Allow server to initialize

    println!("Starting client");
    let mut client = common::start_client();
    thread::sleep(Duration::from_secs(1)); // Allow client to connect

    println!("Sending shutdown signal (SIGINT)");
    signal::kill(Pid::from_raw(server.id() as i32), Signal::SIGINT)
        .expect("Failed to send SIGINT to server");
    thread::sleep(Duration::from_secs(2)); // Allow server to perform cleanup

    println!("Checking if shared memory exists");
    let shm_path = "/dev/shm/RequestQueue";
    assert!(!Path::new(shm_path).exists(), "Shared memory was not cleaned up");

    println!("Killing client");
    client.kill().expect("Failed to kill client");
} 