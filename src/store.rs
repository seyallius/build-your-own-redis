//! <this needs proper description and correct filename or correct
//! module location?

use std::{collections::HashMap, sync::Arc};

// everything below needs docstring

pub(crate) type Key = Vec<u8>;
pub(crate) type Value = Vec<u8>;
pub(crate) type Store = Arc<HashMap<Key, Value>>;
