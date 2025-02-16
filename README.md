# shared_serve
A multi-threaded server which uses shared memory for communication with clients. The server manages a hash table in shared memory. Clients can enqueue requests to the server for operations on the hash table. A dead-lock free algorithm is used to manage access to the shared memory. Complentary [integration tests](tests) are provided to test the server functionality.

## Directory Structure
- [src](src): Source code for the project
  - [main.rs](src/main.rs): Defines the server implementation. This creates shared memory segment and starts waiting for client to enqueue requests.
  - [client.rs](src/client.rs): Defines the client implementation.
  - [lib.rs](src/lib.rs): Defines the hash table [implementation](src/lib.rs#L100) and `Request` [data structure](src/lib.rs#L40).
- [tests](tests): Integration tests for testing various scenarios. More details can be found in [Testing](#testing) section.
- [Cargo.toml](Cargo.toml): Rust project configuration.

## Setup
> [!IMPORTANT]
Make sure to have Rust installed. All the binaries present are tested with `Rust 1.84.1` on `Ubuntu 22.04.3: jammy`.


### Running the server
Server allows specifying following optional command line arguments:
- `--size <size>`: Size of the hash table. **Default is 10.**
- `--threads <num_threads>`: Number of threads to perform concurrent operations on the hash table. **Default is 4.**

```bash
cargo run --bin server -- --size <size> --threads <num_threads>
```
> [!NOTE]
> Server uses a [`CAPACITY` constant](src/lib.rs#L21) to determine the size of requests queue. Change this constant based on the needs.

### Running the client
Client allows two modes of operation:
- `interactive`: Client will prompt for requests and display the response. This is the default mode.
- `stress-test`: Client will keep reading requests from the stdin and will keep enqueing them to the shared memory. Pass `--stress-test` to enable this mode.

> [!WARNING]
> Make sure to conform to the format of [expected input](src/client.rs#L131) while using `stress-test` mode.

```bash
cargo run --bin client [-- --stress-test]
```

## Testing

Unit tests are present in [src/lib.rs](src/lib.rs#L174) for testing the hash table implementation. Integration tests are present in [tests](tests) directory for performing end-to-end testing. 

All the unit and integration tests can be run with:

```bash
cargo test
```
### [Tests](tests) directory presents following tests for checking different aspects of the server:

- [end_to_end_tests.rs](tests/end_to_end_tests.rs): Tests the end-to-end flow of processing requests.  

- [graceful_shutdown_tests.rs](tests/graceful_shutdown_tests.rs): Tests the server's ability to handle abrupt shutdown signal by freeing up the shared memory and exiting gracefully.
    > [!NOTE]
    > Server currently supports a primitive form of graceful shutdown. It currently supports `SIGINT` (Ctrl+C) signal to trigger shutdown.

- [queue_full_tests.rs](tests/queue_full_tests.rs): Tests the scenario where the shared queue of requests becomes full.
    > [!NOTE]
    > This test might block for some time to get file lock on package cache/build directory. This is side-effect of how client is being built and can be ignored.

- [fault_tolerance_tests.rs](tests/fault_tolerance_tests.rs): Tests the server's ability to handle faults on the client side.

### Running a specific test
Issue the following command to run a specific test by changing the test name to desired test name:

```bash
cargo test --test <test_name> -- --nocapture
```
Possible test names are:
- `end_to_end_tests`
- `graceful_shutdown_tests`
- `queue_full_tests`
- `fault_tolerance_tests`



