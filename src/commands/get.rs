//! Implementation of the `GET` command.

use crate::{commands::Command, resp::parser::Value, store};
use anyhow::{Error, Result};

// ------------------------------------- Public (crate) API ------------------------------------- //

pub(crate) struct Get;
impl Command for Get {
    /// Returns the command's canonical name.
    fn name(&self) -> &'static str {
        "get"
    }

    fn execute(&self, args: &[Value], storage: Option<&store::Store>) -> Result<Value> {
        let Some(storage) = storage else {
            return Ok(Value::Error("How's storage map none?!".into()));
        };

        match args {
            [Value::Bulk(Some(key))] => {
                let storage = storage
                    .lock()
                    .map_err(|poison_error| Error::msg(poison_error.to_string()))?;
                if let Some(val) = storage.get(key) {
                    return Ok(Value::Bulk(Some(val.clone())));
                }
                Ok(Value::Bulk(None))
            }
            _ => Ok(Value::Error(
                "ERR wrong number of arguments for 'get' command".into(),
            )),
        }
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[cfg(test)]
mod get_tests {
    use super::*;
    use std::{collections::HashMap, sync::Mutex};

    fn new_store() -> store::Store {
        store::Store::new(Mutex::new(HashMap::new()))
    }

    #[test]
    fn returns_error_without_arguments() {
        let result = Get.execute(&[], None).unwrap();

        assert!(
            matches!(result, Value::Error(ref message) if message == "How's storage map none?!")
        );
    }

    #[test]
    fn gets_simple_string_value() {
        let storage = new_store();
        storage
            .lock()
            .unwrap()
            .insert(b"foo".to_vec(), b"bar".to_vec());

        let args = [Value::Bulk(Some(b"foo".to_vec()))];
        let result = Get.execute(&args, Some(&storage)).unwrap();

        assert!(matches!(result, Value::Bulk(Some(value)) if value == b"bar"));
        let storage = storage.lock().unwrap();
        let stored_val = storage.get(b"foo".as_slice()).unwrap();
        assert_eq!(stored_val, b"bar");
    }

    #[test]
    fn returns_null_for_missing_key() {
        let storage = new_store();
        let args = vec![Value::Bulk(Some(b"foo".to_vec()))];
        let result = Get.execute(&args, Some(&storage)).unwrap();

        assert!(
            matches!(result, Value::Bulk(None)),
            "expected null bulk, got {:?}",
            result
        );
        let storage = storage.lock().unwrap();
        let stored_val = storage.get(b"foo".as_slice());
        assert!(stored_val.is_none(), "key shouldn't exist");
    }

    #[test]
    fn returns_error_when_given_too_many_arguments() {
        let args = [
            Value::Bulk(Some(b"first".to_vec())),
            Value::Bulk(Some(b"second".to_vec())),
            Value::Bulk(Some(b"third".to_vec())),
        ];

        let result = Get.execute(&args, Some(&new_store())).unwrap();

        assert!(matches!(result, Value::Error(_)));
    }
}
