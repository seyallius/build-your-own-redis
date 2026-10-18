//! Shared in-memory storage for Redis keys and values.
//!
//! Keys and values are kept as bytes so the store can hold binary data. The
//! map is protected by a mutex and shared through an [`Arc`], allowing all
//! client connection threads to access the same data.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// A key stored as arbitrary bytes.
pub(crate) type Key = Vec<u8>;

/// A value stored as arbitrary bytes.
pub(crate) type Value = Vec<u8>;

/// The shared, thread-safe map from keys to values.
pub(crate) type Store = Arc<Mutex<HashMap<Key, Value>>>;
