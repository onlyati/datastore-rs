//! Custom types

use std::collections::BTreeMap;

use crate::enums::{KeyType, ValueType};

pub type Table = BTreeMap<KeyType, ValueType>;
