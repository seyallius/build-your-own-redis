//! Shared in-memory store for Redis keys and values.
//!
//! Keys and values are kept as bytes so the store can hold binary data. The
//! map is protected by a mutex and shared through an [`Arc`], allowing all
//! client connection threads to access the same data.

use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

// ------------------------------------------ Types & Impls ------------------------------------- //

/// A key stored as arbitrary bytes.
pub(crate) type Key = Vec<u8>;

/// A value stored as arbitrary bytes.
pub(crate) type Value = Vec<u8>;

/// A borrowed Redis key.
pub(crate) type KeyRef<'a> = &'a [u8];

/// A borrowed Redis value.
pub(crate) type ValueRef<'a> = &'a [u8];

#[derive(Clone, Default, Debug)]
pub(crate) struct Entry {
    value: Value,
    deadline: Option<Instant>,
}
impl Entry {
    // -------------------------------------- Internal Helpers -------------------------------------- //

    fn is_expired_at(&self, now: Instant) -> bool {
        self.deadline.is_some_and(|d| now >= d)
    }

    /// Checks whether a deadline has reached.
    fn is_expired(&self) -> bool {
        self.is_expired_at(Instant::now())
    }
}

/// The actual in-memory map storing byte [`Key`] and byte [`Value`].
pub(crate) type Map = HashMap<Key, Entry>;

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
    pub(crate) fn insert(
        &self,
        key: KeyRef,
        value: ValueRef,
        expiration: Option<Duration>,
    ) -> Result<()> {
        let deadline = expiration
            .map(|duration| {
                Instant::now()
                    .checked_add(duration)
                    .ok_or_else(|| anyhow::anyhow!("expiration deadline overflow"))
            })
            .transpose()?;

        let entry = Entry {
            value: value.to_vec(),
            deadline,
        };

        let mut entries = self.lock_entries()?;
        entries.insert(key.to_vec(), entry);

        Ok(())
    }

    /// Returns a copy of the value associated with `key`, if it exists.
    pub(crate) fn get(&self, key: KeyRef) -> Result<Option<Value>> {
        let mut entries = self.lock_entries()?;
        match entries.get(key) {
            None => Ok(None),
            Some(entry) if entry.is_expired() => {
                entries.remove(key);
                Ok(None)
            }
            Some(entry) => Ok(Some(entry.value.clone())),
        }
    }

    /// Removes a key from the entries, returning the removed value.
    pub(crate) fn remove(&self, key: KeyRef) -> Result<Option<Entry>> {
        let mut entries = self.lock_entries()?;
        Ok(entries.remove(key))
    }

    // -------------------------------------- Internal Helpers -------------------------------------- //

    /// Locks the store and returns a guard for accessing its entries.
    ///
    /// The lock is released when the returned guard is dropped.
    fn lock_entries(&self) -> Result<MutexGuard<'_, Map>> {
        self.entries
            .lock()
            .map_err(|poison_error| anyhow::anyhow!("{poison_error}"))
    }
}

// -------------------------------------------- Tests ------------------------------------------- //

#[cfg(test)]
mod store_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn insert_then_get() {
        let store = Store::new();
        store.insert(b"k", b"v", None).unwrap();
        assert_eq!(store.get(b"k").unwrap(), Some(b"v".to_vec()));
    }

    #[test]
    fn get_missing_is_none() {
        let store = Store::new();
        assert_eq!(store.get(b"nope").unwrap(), None);
    }

    #[test]
    fn insert_replaces() {
        let store = Store::new();
        store.insert(b"k", b"v1", None).unwrap();
        store.insert(b"k", b"v2", None).unwrap();
        assert_eq!(store.get(b"k").unwrap(), Some(b"v2".to_vec()));
    }

    #[test]
    fn remove_returns_once() {
        let store = Store::new();
        store.insert(b"k", b"v", None).unwrap();
        assert!(store.remove(b"k").unwrap().is_some());
        assert!(store.remove(b"k").unwrap().is_none());
    }

    #[test]
    fn cloned_store_shares_data() {
        let a = Store::new();
        let b = a.clone();
        a.insert(b"k", b"v", None).unwrap();
        assert_eq!(b.get(b"k").unwrap(), Some(b"v".to_vec()));
    }

    #[test]
    fn expired_entry_is_not_returned_and_is_purged() {
        let store = Store::new();
        store.insert(b"k", b"v", Some(Duration::ZERO)).unwrap();
        assert_eq!(store.get(b"k").unwrap(), None);
        // purged, not merely hidden:
        assert!(store.remove(b"k").unwrap().is_none());
    }

    #[test]
    fn unexpired_entry_is_returned() {
        let store = Store::new();
        store
            .insert(b"k", b"v", Some(Duration::from_secs(60)))
            .unwrap();
        assert_eq!(store.get(b"k").unwrap(), Some(b"v".to_vec()));
    }

    #[test]
    fn entry_is_expired_at_uses_provided_instant() {
        let now = Instant::now();
        let e = Entry {
            value: vec![],
            deadline: Some(now),
        };
        assert!(e.is_expired_at(now));
        assert!(!e.is_expired_at(now - Duration::from_millis(1)));
        assert!(
            !Entry {
                value: vec![],
                deadline: None
            }
            .is_expired_at(now)
        );
    }
}
