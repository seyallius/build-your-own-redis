//! get - Integration tests for the Redis GET command.

mod common;

use common::{TestClient, TestServer};

#[test]
fn get_returns_stored_value() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    assert_eq!(
        client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$5\r\nhello\r\n"),
        b"+OK\r\n"
    );

    let response = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(response, b"$5\r\nhello\r\n");
}

#[test]
fn get_returns_null_bulk_string_for_missing_key() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response = client.execute(b"*2\r\n$3\r\nGET\r\n$7\r\nmissing\r\n");

    assert_eq!(response, b"$-1\r\n");
}

#[test]
fn get_returns_empty_bulk_string_for_empty_value() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    assert_eq!(
        client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$0\r\n\r\n"),
        b"+OK\r\n"
    );

    let response = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(response, b"$0\r\n\r\n");
}

#[test]
fn get_returns_value_written_by_another_client() {
    let server = TestServer::start();

    let mut writer = TestClient::connect(&server);

    assert_eq!(
        writer.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"),
        b"+OK\r\n"
    );

    drop(writer);

    let mut reader = TestClient::connect(&server);

    let response = reader.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(response, b"$3\r\nbar\r\n");
}

/// Verifies that Redis preserves binary values without UTF-8 conversion.
#[test]
fn get_preserves_binary_values() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    assert_eq!(
        client.execute(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\n\xff\x00\xfe\r\n"),
        b"+OK\r\n"
    );

    let response = client.execute(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

    assert_eq!(response, b"$3\r\n\xff\x00\xfe\r\n");
}
