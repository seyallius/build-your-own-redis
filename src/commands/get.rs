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
    fn execute(&self, args: &[Value], store: Option<&store::Store>) -> Result<Value> {
        let Some(store) = store else {
            return Ok(Value::Error("How's store map none?!".into()));
        };

        let [key] = args else {
            return Ok(Value::error(
                "ERR wrong number of arguments for 'get' command",
            ));
        };

        let Some(key) = key.as_bulk() else {
            return Ok(Value::error("ERR invalid argument type"));
        };

        //optimize(allocation): However, your current `Store::get()` accepts `&Vec<u8>`,
        // forcing an unnecessary allocation with `key.to_vec()`.
        // I would eventually change that API to accept `&[u8]`, since `HashMap<Vec<u8>, Entry>`
        // supports borrowed slice lookups.
        let key = key.to_vec();
        Ok(match store.get(&key)? {
            Some(value) => Value::bulk(value),
            None => Value::null_bulk(),
        })
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

        assert!(matches!(result, Value::Error(ref message) if message == "How's store map none?!"));
    }

    #[test]
    fn gets_simple_string_value() {
        let store = new_store();
        store.insert(&b"foo".to_vec(), &b"bar".to_vec()).unwrap();

        let args = [Value::Bulk(Some(b"foo".to_vec()))];
        let result = Get.execute(&args, Some(&store)).unwrap();

        assert!(matches!(result, Value::Bulk(Some(value)) if value == b"bar"));
        let stored_val = store.get(&b"foo".to_vec()).unwrap().unwrap();
        assert_eq!(stored_val, b"bar");
    }

    #[test]
    fn returns_null_for_missing_key() {
        let store = new_store();
        let args = vec![Value::Bulk(Some(b"foo".to_vec()))];
        let result = Get.execute(&args, Some(&store)).unwrap();

        assert!(
            matches!(result, Value::Bulk(None)),
            "expected null bulk, got {:?}",
            result
        );
        let stored_val = store.get(&b"foo".to_vec()).unwrap();
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
