//! Implementation of the `ECHO` command.

use crate::{commands::Command, resp::parser::Value};
use anyhow::Result;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `ECHO` command: returns its single argument unchanged.
pub(crate) struct Echo;
impl Command for Echo {
    // -------------------------------------- Internal Helpers -------------------------------------- //

    fn name(&self) -> &'static str {
        "echo"
    }

    fn execute(&self, args: &[Value]) -> Result<Value> {
        match args {
            [Value::Bulk(Some(msg))] => Ok(Value::Bulk(Some(msg.clone()))),
            _ => Ok(Value::Error("wrong number of arguments for 'echo'".into())),
        }
    }
}
