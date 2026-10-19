//! set - Integration tests for the Redis SET command.

mod common;

use std::{thread, time::Duration};

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

#[test]
fn set_with_px_expires_after_duration() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response =
        client.execute(b"*5\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$2\r\nPX\r\n$3\r\n100\r\n");

    thread::sleep(Duration::from_millis(200));
    let value = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(response, b"+OK\r\n");
    assert_eq!(value, b"$-1\r\n");
}

#[test]
fn set_with_lowercase_px_expires_after_duration() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response =
        client.execute(b"*5\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$2\r\npx\r\n$3\r\n100\r\n");

    thread::sleep(Duration::from_millis(200));
    let value = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(response, b"+OK\r\n");
    assert_eq!(value, b"$-1\r\n");
}

#[test]
fn set_without_expiry_remains_available() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response = client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");

    thread::sleep(Duration::from_millis(200));
    let value = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(response, b"+OK\r\n");
    assert_eq!(value, b"$3\r\nbar\r\n");
}

#[test]
fn overwriting_expiring_key_without_px_removes_expiry() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let set_with_px =
        client.execute(b"*5\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$2\r\nPX\r\n$3\r\n100\r\n");

    let set = client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbaz\r\n");

    thread::sleep(Duration::from_millis(200));

    let value = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(set_with_px, b"+OK\r\n");
    assert_eq!(set, b"+OK\r\n");
    assert_eq!(value, b"$3\r\nbaz\r\n");
}
