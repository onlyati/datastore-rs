//! Custom types

use std::collections::BTreeMap;
use crate::enums::ErrorKind;

use crate::enums::{KeyType, ValueType};

pub type Table = BTreeMap<KeyType, ValueType>;

pub type ResultWithResult = Result<ValueType, ErrorKind>;
pub type ResultWithoutResult = Result<(), ErrorKind>;
pub type ResultWithList = Result<Vec<KeyType>, ErrorKind>;
