#![allow(dead_code)]
use std::process::{Command, Child, Stdio};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::io::BufReader;
use std::io::BufRead;
use std::time::Instant;
use std::time::Duration;
use std::process::ChildStdout;

pub const BUCKET_COUNT: usize = 10;

pub fn start_server() -> Child {
    Command::new("cargo")
        .args(&["run", "--bin", "server", "--", "--size", &BUCKET_COUNT.to_string()])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start server")
}

pub fn start_client() -> Child {
    Command::new("cargo")
        .args(&["run", "--bin", "client", "--", "--stress-test"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start client")
}

pub fn stop_server_with_sigint(server: &Child) {
    // Send SIGINT to the server
    signal::kill(Pid::from_raw(server.id() as i32), Signal::SIGINT)
        .expect("Failed to send SIGINT to server");
}


pub fn check_server_output(server_output: ChildStdout, expected_output: Vec<&str>, timeout: Duration)  {
    let mut bufread = BufReader::new(server_output);
    let mut buf = String::new();
    let mut found_lines = vec![false; expected_output.len()];
    // Time out after 10 seconds
    let start_time = Instant::now();

    while let Ok(n) = bufread.read_line(&mut buf) {
        if n == 0 { break; }
        
        if start_time.elapsed() > timeout {
            panic!("Server output timed out after {:?} seconds", timeout);
        }
        // Check each expected line
        for (i, expected) in expected_output.iter().enumerate() {
            if buf.contains(expected) {
                found_lines[i] = true;
            }
        }

        if found_lines.iter().all(|&x| x) {
            break;
        }

        buf.clear();
    }

    // Assert all lines were found
    for (i, expected) in expected_output.iter().enumerate() {
        assert!(found_lines[i], "Expected output '{}' not found", expected);
    }
}
