//! Enum for the crate

use std::sync::mpsc::Sender;
use std::cmp::Ordering;
use std::fmt::Display;
use crate::types::Table;

///
/// Possible error types that database can return
/// 
#[derive(Debug)]
pub enum ErrorKind {
    /// The root name in the key does not match with the root table name
    InvalidRoot(String),

    /// Wrong key is specified, reason in the message
    InvalidKey(String),

    /// Oops, it should not happen
    InternalError(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let response = match self {
            Self::InvalidKey(message) => format!("Invalid key: {message}"),
            Self::InvalidRoot(message) => format!("Invalid root: {message}"),
            Self::InternalError(message) => format!("Internal error: {message}"),
        };
        return write!(f, "{}", response);
    }
}

///
/// Key type that database accept, it can be record or another table
/// 
#[derive(Eq, Ord, Debug, Clone)]
pub enum KeyType {
    /// Value will be a pointer to another table
    Table(String),

    /// Value will be a string
    Record(String),
}

impl KeyType {
    /// Tells that key type is `KeyType::Table`
    pub fn is_table(&self) -> bool {
        return match self {
            KeyType::Record(_) => false,
            KeyType::Table(_) => true,
        };
    }

    /// Tells that key type is `KeyType::Record`
    pub fn is_record(&self) -> bool {
        return !self.is_table();
    }

    /// Return with the record name or the table name
    pub fn get_key(&self) -> &String {
        return match self {
            KeyType::Record(key) => key,
            KeyType::Table(key) => key,
        };
    }

    /// Show type as string
    pub fn get_type(&self) -> &str {
        return match self {
            KeyType::Record(_) => "r",
            KeyType::Table(_) => "t",
        };
    } 
}

impl Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Table(key) => ("t", key),
            Self::Record(key) => ("r", key),
        };
        return write!(f, "{} {}", message.0, message.1);
    }
}

impl PartialOrd for KeyType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_key = self.get_key();
        let other_key = self.get_key();

        if self_key == other_key {
            if self.is_table() && other.is_record() {
                return Some(Ordering::Greater);
            } else if self.is_record() && other.is_table() {
                return Some(Ordering::Less);
            } else {
                return Some(Ordering::Equal);
            }
        }

        return Some(self_key.cmp(other_key));
    }
}

impl PartialEq for KeyType {
    fn eq(&self, other: &Self) -> bool {
        if (self.is_record() && other.is_record()) || (self.is_table() && other.is_table()) {
            if self.get_key() == other.get_key() {
                return true;
            }
        }
        return false;
    }
}

///
/// Specifiy the level for listing key function
/// 
#[derive(PartialEq, Clone)]
pub enum ListType {
    /// List only the current level
    OneLevel,

    /// List everything under it on recursive way
    All,
}

/// Type of the value
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// This is a table pointer, belongs to `KeyType::Table`
    TablePointer(Table),

    /// This is a record pointer, belongs to `KeyType::Record`
    RecordPointer(String),
}

impl ValueType {
    /// Tells that it is a `ValueType::TablePointer`
    pub fn is_table(&self) -> bool {
        return match self {
            ValueType::RecordPointer(_) => false,
            ValueType::TablePointer(_) => true,
        };
    }

    /// Tells that it is a `ValueType::RecordPointer`
    pub fn is_record(&self) -> bool {
        return !self.is_table();
    }
}

///
/// Actions for built-in server
/// 
pub enum DatabaseAction {
    /// Set or update a key-value pair
    Set(Sender<Result<(), ErrorKind>>, String, String),

    /// Get a value for a key
    Get(Sender<Result<ValueType, ErrorKind>>, String),

    /// Delete a pair
    DeleteKey(Sender<Result<(), ErrorKind>>, String),

    /// Delete a whole table
    DeleteTable(Sender<Result<(), ErrorKind>>, String),

    /// List keys from a route
    ListKeys(Sender<Result<Vec<KeyType>, ErrorKind>>, String, ListType),
}
