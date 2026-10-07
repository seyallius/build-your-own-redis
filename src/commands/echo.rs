//! Implementation of the `ECHO` command.

use crate::{commands::Command, resp::parser::Value};
use anyhow::Result;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `ECHO` command: returns its single argument unchanged.
pub(crate) struct Echo;
impl Command for Echo {
    /// Returns the command's canonical name.
    fn name(&self) -> &'static str {
        "echo"
    }

    /// Copies the argument bytes into the response value.
    ///
    /// The bytes stay binary-safe: they are not converted to UTF-8 text.
    fn execute(&self, args: &[Value]) -> Result<Value> {
        match args {
            [Value::Bulk(Some(msg))] => Ok(Value::Bulk(Some(msg.clone()))),
            _ => Ok(Value::Error(
                "ERR wrong number of arguments for 'echo' command".into(),
            )),
        }
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[cfg(test)]
mod echo_tests {
    use super::*;

    #[test]
    fn echoes_one_bulk_string_argument() {
        let args = [Value::Bulk(Some(b"hello".to_vec()))];

        let result = Echo.execute(&args).unwrap();

        assert!(matches!(
            result,
            Value::Bulk(Some(bytes)) if bytes.as_slice() == b"hello"
        ));
    }

    #[test]
    fn returns_error_when_argument_is_missing() {
        let result = Echo.execute(&[]).unwrap();

        assert!(matches!(result, Value::Error(_)));
    }

    #[test]
    fn returns_error_when_argument_is_not_a_bulk_string() {
        let args = [Value::Simple("hello".into())];

        let result = Echo.execute(&args).unwrap();

        assert!(matches!(result, Value::Error(_)));
    }
}
