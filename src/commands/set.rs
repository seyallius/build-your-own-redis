//! Implementation of the `SET` command.

use crate::{commands::Command, resp::parser::Value, store};
use anyhow::{Error, Result};

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
    fn execute(&self, args: &[Value], storage: Option<&store::Store>) -> Result<Value> {
        let Some(storage) = storage else {
            return Ok(Value::Error("How's storage map none?!".into()));
        };

        match args {
            [Value::Bulk(Some(key)), Value::Bulk(Some(val))] => {
                let mut storage = storage
                    .lock()
                    .map_err(|poison_error| Error::msg(poison_error.to_string()))?;
                storage.insert(key.to_vec(), val.to_vec());
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
    use std::{collections::HashMap, sync::Mutex};

    fn new_store() -> store::Store {
        store::Store::new(Mutex::new(HashMap::new()))
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
        let storage = new_store();
        let result = Set.execute(&args, Some(&storage)).unwrap();

        assert!(matches!(result, Value::Simple(ref message) if message == "OK"));
        let storage = storage.lock().unwrap();
        let stored_val = storage.get(b"foo".as_slice()).unwrap();
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
