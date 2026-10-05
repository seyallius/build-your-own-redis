//! Stub implementation of the `COMMAND` command.
//!
//! `redis-cli` issues `COMMAND DOCS` on connect to discover server
//! capabilities. We do not implement any introspection yet, so we satisfy
//! the client by returning an empty array. Replace this with a real reply
//! when command metadata is added.

use crate::{commands::Command, resp::parser::Value};
use anyhow::Result;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `COMMAND` command (intentionally a no-op).
pub(crate) struct CommandCmd;
impl Command for CommandCmd {
    // -------------------------------------- Internal Helpers -------------------------------------- //

    fn name(&self) -> &'static str {
        "command"
    }

    fn execute(&self, _args: &[Value]) -> Result<Value> {
        Ok(Value::Array(Vec::new()))
    }
}
