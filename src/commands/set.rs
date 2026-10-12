//! Implementation of the `SET` command.

use crate::{commands::Command, resp::parser::Value, store};
use anyhow::Result;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `SET` command.
///
/// * `SET` replies with the simple string `PONG`.
/// * `SET <message>` replies with `<message>` as a bulk string.
///
/// This matches the behavior of Redis 2.8+.
pub(crate) struct Set;
impl Command for Set {
    /// Returns the command's canonical name.
    fn name(&self) -> &'static str {
        "set"
    }

    /// Returns `PONG` with no argument, or echoes one supplied argument.
    fn execute(&self, args: &[Value], storage: Option<&store::Store>) -> Result<Value> {
        let Some(storage) = storage else {
            return Ok(Value::Error("How's storage map none?!".into()));
        };

        match args {
            [Value::Bulk(Some(key)), Value::Bulk(Some(val))] => {
                dbg!(str::from_utf8(key).expect("key what?"));
                dbg!(str::from_utf8(val).expect("val what?"));
                Ok(Value::Simple(String::from("OK")))
            }
            _ => Ok(Value::Error(
                "ERR wrong number of arguments for 'set' command".into(),
            )),
        }
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[cfg(test)]
mod set_tests {
    use super::*;
    use std::collections::HashMap;

    fn new_store() -> store::Store {
        store::Store::new(HashMap::new())
    }

    #[test]
    fn returns_error_without_arguments() {
        let result = Set.execute(&[], None).unwrap();

        assert!(
            matches!(result, Value::Error(ref message) if message == "How's storage map none?!")
        );
    }

    #[test]
    fn sets_simple_string_value() {
        let args = [
            Value::Bulk(Some(b"foo".to_vec())),
            Value::Bulk(Some(b"bar".to_vec())),
        ];
        let result = Set.execute(&args, Some(&new_store())).unwrap();

        assert!(matches!(result, Value::Simple(ref message) if message == "OK"));
    }

    #[test]
    fn returns_error_when_given_too_many_arguments() {
        let args = [
            Value::Bulk(Some(b"first".to_vec())),
            Value::Bulk(Some(b"second".to_vec())),
            Value::Bulk(Some(b"third".to_vec())),
        ];

        let result = Set.execute(&args, Some(&new_store())).unwrap();

        assert!(matches!(result, Value::Error(_)));
    }
}
