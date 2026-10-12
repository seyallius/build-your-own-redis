//! Integration test for `echo` command.

mod common;

use common::{TestClient, TestServer};

#[test]
fn echo_returns_argument_as_resp_bulk_string() {
    let server = TestServer::start();
    let mut client = TestClient::connect(&server);

    let response = client.execute(b"*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n");

    assert_eq!(response, b"$5\r\nhello\r\n");
}
