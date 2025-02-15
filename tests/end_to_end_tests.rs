use std::process::{Child, ChildStdout, Command, Stdio};
use std::thread;
use std::time::Duration;
use std::io::Write;
mod common;


#[test]
fn test_end_to_end_request_handling() {
    let mut server = common::start_server();
    thread::sleep(Duration::from_secs(2));

    let mut client = common::start_client();
    thread::sleep(Duration::from_secs(1));

    if let Some(mut client_stdin) = client.stdin.as_mut() {
        writeln!(client_stdin, "INSERT test_key test_value").unwrap();
        writeln!(client_stdin, "GET test_key").unwrap();
        writeln!(client_stdin, "DELETE test_key").unwrap();
        writeln!(client_stdin, "exit").unwrap();

        client_stdin.flush().expect("Failed to flush stdin");
    }

    thread::sleep(Duration::from_secs(2));
    
    let expected_output = vec!["Inserting key: test_key", "Getting key: test_key", "Value: test_value", "Deleting key: test_key"];
    common::check_server_output(server.stdout.take().unwrap(), expected_output, Duration::from_secs(60));

    // Cleanup
    common::stop_server_with_sigint(&server);
    server.wait().expect("Failed to wait for server to exit");
    client.wait().expect("Failed to kill client");
} 