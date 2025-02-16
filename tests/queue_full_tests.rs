use std::thread;
use std::time::Duration;
mod common;
use std::io::Write;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

const CAPACITY: usize = 10;

#[test]
fn test_queue_full_handling() {
    let mut server = common::start_server();
    thread::sleep(Duration::from_secs(1));
    
    // Stop the server to simulate a congestion
    signal::kill(Pid::from_raw(server.id() as i32), Signal::SIGSTOP)
        .expect("Failed to send SIGSTOP to server");
    println!("Server stopped");

    let mut clients = Vec::new();

    for i in 0..(CAPACITY + 1) {
        let mut client = common::start_client();
        if let Some(client_stdin) = client.stdin.as_mut() {
            writeln!(client_stdin, "INSERT key{} value{}", i, i).unwrap();
            writeln!(client_stdin, "exit").unwrap();
            client_stdin.flush().expect("Failed to flush stdin");
        }
        clients.push(client);
    }

    let mut queue_full_count = 0;
    let mut client_id = 0;
    for client in clients {
        println!("Client {}:", client_id);
        let output = client.wait_with_output().expect("Failed to get client output");
        if String::from_utf8_lossy(&output.stdout).contains("Queue is full") {
            queue_full_count += 1;
        }
        client_id += 1;
    }

    assert!(queue_full_count > 0, "No clients reported queue full errors");
    
    signal::kill(Pid::from_raw(server.id() as i32), Signal::SIGCONT)
    .expect("Failed to send SIGCONT to server");
    println!("Server resumed");

    common::stop_server_with_sigint(&server);
    server.wait().expect("Failed to wait for server to exit");
} 