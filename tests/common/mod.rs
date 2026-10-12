//! common - Shared Redis integration test infrastructure.

use std::{
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// Maximum time waiting for the Redis server to start.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(2);
/// Maximum time a socket read may wait for a response.
const READ_TIMEOUT: Duration = Duration::from_secs(2);
/// Duration between each retry.
const RETRY_INTERVAL: Duration = Duration::from_millis(10);

/// ------------------------------------------ Types & Impls ------------------------------------- //

/// Owns a Redis server process and terminates it on drop.
pub struct TestServer {
    child: Child,
    address: SocketAddr,
}
impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl TestServer {
    // ----------------------------------------- Public API ----------------------------------------- //

    /// Starts Redis on a dynamically allocated TCP port.
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("could not allocate test port");

        let address = listener
            .local_addr()
            .expect("could not determine test address");

        drop(listener);

        let binary = env::var("CARGO_BIN_EXE_build-your-own-redis")
            .expect("could not find build-your-own-redis binary");
        let child = Command::new(binary)
            .args(["--port", &address.port().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("could not start Redis server");

        let mut server = Self { child, address };
        server.wait_until_ready();

        server
    }

    /// Returns the server's listening address.
    pub fn address(&self) -> SocketAddr {
        self.address
    }
}
impl TestServer {
    // -------------------------------------- Internal Helpers -------------------------------------- //

    /// Waits until the server accepts TCP connections.
    fn wait_until_ready(&mut self) {
        let deadline = Instant::now() + STARTUP_TIMEOUT;

        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .expect("could not check server process")
            {
                panic!("server exited before accepting connections: {status}");
            }

            if TcpStream::connect(self.address).is_ok() {
                return;
            }

            assert!(
                Instant::now() < deadline,
                "server did not start before the timeout"
            );

            thread::sleep(RETRY_INTERVAL);
        }
    }
}

/// Provides a TCP connection to the test Redis server.
pub struct TestClient {
    stream: TcpStream,
}
impl TestClient {
    // ----------------------------------------- Public API ----------------------------------------- //

    /// Connects to the running Redis server.
    pub fn connect(server: &TestServer) -> Self {
        let stream = TcpStream::connect(server.address()).expect("could not connect to Redis");

        stream
            .set_read_timeout(Some(READ_TIMEOUT))
            .expect("could not set socket read timeout");

        Self { stream }
    }

    /// Sends a raw RESP request and reads its complete response.
    pub fn execute(&mut self, request: &[u8]) -> Vec<u8> {
        self.stream
            .write_all(request)
            .expect("could not send Redis request");

        self.read_response()
    }
}
impl TestClient {
    // -------------------------------------- Internal Helpers -------------------------------------- //

    /// Reads one RESP simple string, error, integer, or bulk string.
    fn read_response(&mut self) -> Vec<u8> {
        let mut response = Vec::new();
        let mut byte = [0_u8; 1];

        self.stream
            .read_exact(&mut byte)
            .expect("could not read RESP type");

        response.push(byte[0]);

        match byte[0] {
            b'+' | b'-' | b':' => {
                self.read_line(&mut response);
            }
            b'$' => {
                self.read_line(&mut response);

                let header = std::str::from_utf8(&response[1..response.len() - 2])
                    .expect("invalid bulk string length");

                let length: i64 = header.parse().expect("invalid bulk string length");

                if length >= 0 {
                    let length = usize::try_from(length).expect("bulk string length exceeds usize");

                    let start = response.len();
                    response.resize(start + length + 2, 0);

                    self.stream
                        .read_exact(&mut response[start..])
                        .expect("could not read bulk string payload");
                }
            }
            marker => panic!("unsupported RESP response marker: {marker}"),
        }

        response
    }

    /// Reads bytes through the next CRLF terminator.
    fn read_line(&mut self, response: &mut Vec<u8>) {
        loop {
            let mut byte = [0_u8; 1];

            self.stream
                .read_exact(&mut byte)
                .expect("could not read RESP line");

            response.push(byte[0]);

            if response.ends_with(b"\r\n") {
                return;
            }
        }
    }
}
