//! Small compatibility stub for redis-cli's COMMAND DOCS probe.
//!
//! `redis-cli` issues `COMMAND DOCS` on connect to discover server
//! capabilities. We do not implement any introspection yet, so we satisfy
//! the client by returning an empty array. Replace this with a real reply
//! when command metadata is added.

use crate::{commands::Command, resp::parser::Value};
use anyhow::Result;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `COMMAND` command (intentionally a no-op).
/// Handles the limited COMMAND request sent by redis-cli during startup.
pub(crate) struct CommandCmd;
impl Command for CommandCmd {
    /// Returns the command's canonical name.
    fn name(&self) -> &'static str {
        "command"
    }

    /// Accepts `COMMAND DOCS` and returns an empty array.
    ///
    /// This lets redis-cli continue its startup when this server does not
    /// provide command metadata. It is a compatibility stub, not a full
    /// implementation of Redis' COMMAND command.
    fn execute(&self, _args: &[Value]) -> Result<Value> {
        Ok(Value::Array(Vec::new()))
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn accepts_docs_subcommand() {
        let args = [Value::Bulk(Some(b"DOCS".to_vec()))];

        let result = CommandCmd.execute(&args).unwrap();

        assert!(matches!(result, Value::Array(items) if items.is_empty()));
    }
}
