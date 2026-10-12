//! set - Integration tests for the Redis SET command.

mod common;

use common::{TestClient, TestServer};

#[test]
fn set_returns_ok() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response = client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");

    assert_eq!(response, b"+OK\r\n");
}

#[test]
fn set_overwrites_existing_value() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let first = client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");

    let second = client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbaz\r\n");

    let value = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(first, b"+OK\r\n");
    assert_eq!(second, b"+OK\r\n");
    assert_eq!(value, b"$3\r\nbaz\r\n");
}

#[test]
fn set_stores_multiple_independent_keys() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    assert_eq!(
        client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"),
        b"+OK\r\n"
    );

    assert_eq!(
        client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nbaz\r\n$3\r\nqux\r\n"),
        b"+OK\r\n"
    );

    assert_eq!(
        client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n"),
        b"$3\r\nbar\r\n"
    );

    assert_eq!(
        client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nbaz\r\n"),
        b"$3\r\nqux\r\n"
    );
}
