use std::thread;
use std::time::Duration;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
mod common;

#[test]
fn test_fault_tolerance_on_client_crash() {
    let mut server = common::start_server();
    thread::sleep(Duration::from_secs(2));

    let mut client = common::start_client();
    thread::sleep(Duration::from_secs(1));

    // Simulate client crash
    client.kill().expect("Failed to kill client abruptly");

    thread::sleep(Duration::from_secs(2));

    // Start a new client
    let mut new_client = common::start_client();
    thread::sleep(Duration::from_secs(1));

    if let Some(mut new_client_stdin) = new_client.stdin.as_mut() {
        writeln!(new_client_stdin, "INSERT new_key new_value").unwrap();
        writeln!(new_client_stdin, "exit").unwrap();
        new_client_stdin.flush().expect("Failed to flush stdin");
    }

    thread::sleep(Duration::from_secs(2));

    common::check_server_output(server.stdout.take().unwrap(), vec!["Inserting key: new_key"], Duration::from_secs(10));

    // Cleanup
    new_client.wait().expect("Failed to kill new client");
    common::stop_server_with_sigint(&server);
    server.wait().expect("Failed to wait for server to exit");
} 