//! Implementation of the `GET` command.

use crate::{commands::Command, resp::parser::Value, store};
use anyhow::Result;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// The `GET` command.
///
/// Looks up a key and returns its value as a bulk string. If the key does not
/// exist, it returns a null bulk string.
pub(crate) struct Get;
impl Command for Get {
    /// Returns the command's canonical name.
    fn name(&self) -> &'static str {
        "get"
    }

    /// Looks up one bulk-string key in the shared store.
    ///
    /// Returns an error reply if the arguments have the wrong shape.
    fn execute(&self, args: &[Value], storage: Option<&store::Store>) -> Result<Value> {
        let Some(storage) = storage else {
            return Ok(Value::Error("How's storage map none?!".into()));
        };

        match args {
            [Value::Bulk(Some(key))] => {
                let storage = storage.map()?;
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

    fn new_store() -> store::Store {
        store::Store::new()
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
        let mut map = storage.map().unwrap();
        map.insert(b"foo".to_vec(), b"bar".to_vec());
        drop(map);

        let args = [Value::Bulk(Some(b"foo".to_vec()))];
        let result = Get.execute(&args, Some(&storage)).unwrap();

        assert!(matches!(result, Value::Bulk(Some(value)) if value == b"bar"));
        let map = storage.map().unwrap();
        let stored_val = map.get(b"foo".as_slice()).unwrap();
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
        let storage = storage.map().unwrap();
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
