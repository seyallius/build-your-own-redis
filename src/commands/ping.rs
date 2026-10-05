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
    // -------------------------------------- Internal Helpers -------------------------------------- //

    fn name(&self) -> &'static str {
        "ping"
    }

    fn execute(&self, args: &[Value]) -> Result<Value> {
        match args {
            [] => Ok(Value::Simple("PONG".to_string())),
            [Value::Bulk(Some(msg))] => Ok(Value::Bulk(Some(msg.clone()))),
            _ => Ok(Value::Error("wrong number of arguments for 'ping'".into())),
        }
    }
}
