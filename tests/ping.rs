//! Integration test for `ping` command.

mod common;

use common::{TestClient, TestServer};

#[test]
fn ping_returns_pong() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response = client.execute(b"*1\r\n$4\r\nPING\r\n");

    assert_eq!(response, b"+PONG\r\n");
}

#[test]
fn ping_with_message_returns_message() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response = client.execute(b"*2\r\n$4\r\nPING\r\n$11\r\nhello world\r\n");

    assert_eq!(response, b"$11\r\nhello world\r\n");
}
