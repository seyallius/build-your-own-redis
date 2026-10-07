//! Implementation of the `PING` command.

use crate::{commands::Command, resp::parser::Value};
use anyhow::Result;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `PING` command.
///
/// * `PING` replies with the simple string `PONG`.
/// * `PING <message>` replies with `<message>` as a bulk string.
///
/// This matches the behavior of Redis 2.8+.
pub(crate) struct Ping;
impl Command for Ping {
    /// Returns the command's canonical name.
    fn name(&self) -> &'static str {
        "ping"
    }

    /// Returns `PONG` with no argument, or echoes one supplied argument.
    fn execute(&self, args: &[Value]) -> Result<Value> {
        match args {
            [] => Ok(Value::Simple("PONG".to_string())),
            [Value::Bulk(Some(msg))] => Ok(Value::Bulk(Some(msg.clone()))),
            _ => Ok(Value::Error(
                "ERR wrong number of arguments for 'ping' command".into(),
            )),
        }
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[cfg(test)]
mod ping_tests {
    use super::*;

    #[test]
    fn returns_pong_without_arguments() {
        let result = Ping.execute(&[]).unwrap();

        assert!(matches!(result, Value::Simple(ref message) if message == "PONG"));
    }

    #[test]
    fn echoes_one_bulk_string_argument() {
        let args = [Value::Bulk(Some(b"hello".to_vec()))];

        let result = Ping.execute(&args).unwrap();

        assert!(matches!(
            result,
            Value::Bulk(Some(bytes)) if bytes.as_slice() == b"hello"
        ));
    }

    #[test]
    fn returns_error_when_given_too_many_arguments() {
        let args = [
            Value::Bulk(Some(b"first".to_vec())),
            Value::Bulk(Some(b"second".to_vec())),
        ];

        let result = Ping.execute(&args).unwrap();

        assert!(matches!(result, Value::Error(_)));
    }
}
