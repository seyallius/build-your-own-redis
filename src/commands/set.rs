//! Implementation of the `SET` command.

use crate::{commands::Command, resp::parser::Value, store};
use anyhow::Result;
use std::time::Duration;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `SET` command.
///
/// Stores the supplied value under the supplied key and replies with the
/// simple string `OK`.
pub(crate) struct Set;
impl Command for Set {
    /// Returns the command's canonical name.
    fn name(&self) -> &'static str {
        "set"
    }

    /// Stores one bulk-string value under one bulk-string key.
    ///
    /// Returns an error reply if the arguments have the wrong shape.
    fn execute(&self, args: &[Value], store: Option<&store::Store>) -> Result<Value> {
        let Some(store) = store else {
            return Ok(Value::Error("How's store map none?!".into()));
        };

        let [key, value, options @ ..] = args else {
            return Ok(Value::error(
                "ERR wrong number of arguments for 'set' command",
            ));
        };

        let (Some(key), Some(value)) = (key.as_bulk(), value.as_bulk()) else {
            return Ok(Value::error("ERR invalid argument type"));
        };

        let expiration = match self.parse_expiration(options) {
            Ok(expiration) => expiration,
            Err(error) => return Ok(error),
        };

        store.insert(key, value, expiration)?;
        Ok(Value::simple("OK"))
    }
}
impl Set {
    // -------------------------------------- Internal Helpers -------------------------------------- //

    /// Parses the optional PX expiration argument.
    fn parse_expiration(&self, options: &[Value]) -> std::result::Result<Option<Duration>, Value> {
        match options {
            [] => Ok(None),

            [option, milliseconds]
                if option
                    .as_bulk()
                    .is_some_and(|bytes| bytes.eq_ignore_ascii_case(b"PX")) =>
            {
                let milliseconds = milliseconds
                    .as_bulk()
                    .and_then(|bytes| str::from_utf8(bytes).ok())
                    .and_then(|text| text.parse::<u64>().ok())
                    .filter(|&ms| ms > 0)
                    .ok_or_else(|| Value::error("ERR invalid expire time in 'set' command"))?;

                Ok(Some(Duration::from_millis(milliseconds)))
            }

            _ => Err(Value::error("ERR syntax error")),
        }
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[cfg(test)]
mod set_tests {
    use super::*;

    fn new_store() -> store::Store {
        store::Store::new()
    }

    #[test]
    fn returns_error_without_arguments() {
        let result = Set.execute(&[], None).unwrap();

        assert!(matches!(result, Value::Error(ref message) if message == "How's store map none?!"));
    }

    #[test]
    fn sets_simple_string_value() {
        let args = [
            Value::Bulk(Some(b"foo".to_vec())),
            Value::Bulk(Some(b"bar".to_vec())),
        ];
        let store = new_store();
        let result = Set.execute(&args, Some(&store)).unwrap();

        assert!(matches!(result, Value::Simple(ref message) if message == "OK"));
        let stored_val = store.get(&b"foo".to_vec()).unwrap().unwrap();
        assert_eq!(stored_val, b"bar");
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
