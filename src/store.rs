//! Shared in-memory store for Redis keys and values.
//!
//! Keys and values are kept as bytes so the store can hold binary data. The
//! map is protected by a mutex and shared through an [`Arc`], allowing all
//! client connection threads to access the same data.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Error, Result};

// ------------------------------------------ Types & Impls ------------------------------------- //

/// A key stored as arbitrary bytes.
pub(crate) type Key = Vec<u8>;

/// A value stored as arbitrary bytes.
pub(crate) type Value = Vec<u8>;

/// The actual in-memory map storing byte [`Key`] and byte [`Value`].
pub(crate) type Map = HashMap<Key, Value>;

/// A cloneable handle to shared, thread-safe in-memory store.
#[derive(Clone, Default)]
pub(crate) struct Store {
    /// The shared, thread-safe map from keys to values.
    entries: Arc<Mutex<Map>>,
}
impl Store {
    /// Creates an empty store.
    pub(crate) fn new() -> Self {
        Self::default()
    }
}
impl Store {
    // ------------------------------------- Public (crate) API ------------------------------------- //

    /// Inserts or replaces the value associated with `key`.
    ///
    /// The key and value are copied into the store.
    pub(crate) fn insert(&self, key: &Key, value: &Value) -> Result<()> {
        let mut entries = self.lock_entries()?;
        entries.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    /// Returns a copy of the value associated with `key`, if it exists.
    pub(crate) fn get(&self, key: &Key) -> Result<Option<Value>> {
        let entries = self.lock_entries()?;
        let value = entries.get(key);
        Ok(value.cloned())
    }

    // -------------------------------------- Internal Helpers -------------------------------------- //

    /// Locks the store and returns a guard for accessing its entries.
    ///
    /// The lock is released when the returned guard is dropped.
    fn lock_entries(&self) -> Result<MutexGuard<'_, Map>> {
        self.entries
            .lock()
            .map_err(|poison_error| Error::msg(poison_error.to_string()))
    }
}
