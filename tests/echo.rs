//! Integration test for `echo` command.

use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

// ------------------------------------------ Types & Impls ------------------------------------- //

const SERVER_ADDRESS: &str = "127.0.0.1:6379";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(2);
const RETRY_INTERVAL: Duration = Duration::from_millis(10);

struct RunningServer(Child);
impl Drop for RunningServer {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[test]
fn echo_returns_argument_as_resp_bulk_string() {
    let binary_path = env::var("CARGO_BIN_EXE_build-your-own-redis")
        .expect("Cargo did not provide the server binary path");
    let child = Command::new(binary_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not start the Redis server");

    let mut server = RunningServer(child);
    wait_for_server(&mut server);

    let mut client =
        TcpStream::connect(SERVER_ADDRESS).expect("could not connect to the Redis server");
    client
        .set_read_timeout(Some(STARTUP_TIMEOUT))
        .expect("could not set socket read timeout");

    client
        .write_all(b"*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n")
        .expect("could not send ECHO request");

    let mut response = [0; 11];
    client
        .read_exact(&mut response)
        .expect("could not read complete ECHO response");

    assert_eq!(&response, b"$5\r\nhello\r\n");
}

// -------------------------------------- Internal Helpers -------------------------------------- //

fn wait_for_server(server: &mut RunningServer) {
    let deadline = Instant::now() + STARTUP_TIMEOUT;

    loop {
        if let Some(status) = server.0.try_wait().expect("could not check server process") {
            panic!("server exited before accepting connections: {status}");
        }

        if TcpStream::connect(SERVER_ADDRESS).is_ok() {
            return;
        }

        assert!(
            Instant::now() < deadline,
            "server did not start before the timeout"
        );

        thread::sleep(RETRY_INTERVAL);
    }
}
