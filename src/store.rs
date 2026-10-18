//! Shared in-memory storage for Redis keys and values.
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

/// A cloneable handle to shared, thread-safe in-memory storage.
#[derive(Clone, Default)]
pub(crate) struct Store {
    /// The shared, thread-safe map from keys to values.
    map: Arc<Mutex<Map>>,
}
impl Store {
    /// Creates an empty store.
    pub(crate) fn new() -> Self {
        Self::default()
    }
}
impl Store {
    /// Locks the store and returns a guard for accessing the map;
    /// the lock is released when the guard is dropped.
    pub(crate) fn map(&self) -> Result<MutexGuard<'_, Map>> {
        self.map
            .lock()
            .map_err(|poison_error| Error::msg(poison_error.to_string()))
    }
}
