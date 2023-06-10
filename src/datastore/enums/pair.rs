use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Display;

///
/// Key type that database accept, it can be record or another table
///
#[derive(Eq, Ord, Debug, Clone)]
pub enum KeyType {
    /// Value will be a pointer to another table
    Table(String),

    /// Value will be a string
    Record(String),

    /// Value will be a queue
    Queue(String),
}

impl KeyType {
    /// Tells that key type is `KeyType::Table`
    pub fn is_table(&self) -> bool {
        return match self {
            KeyType::Table(_) => true,
            _ => false,
        };
    }

    /// Tells that key type is `KeyType::Record`
    pub fn is_record(&self) -> bool {
        return match self {
            KeyType::Record(_) => true,
            _ => false,
        };
    }

    /// Tells that key type is `KeyType::Queue`
    pub fn is_queue(&self) -> bool {
        return match self {
            KeyType::Queue(_) => true,
            _ => false,
        };
    }

    /// Return with the record name or the table name
    pub fn get_key(&self) -> &str {
        return match self {
            KeyType::Record(key) => key,
            KeyType::Table(key) => key,
            KeyType::Queue(key) => key,
        };
    }

    /// Show type as string
    pub fn get_type(&self) -> &str {
        return match self {
            KeyType::Record(_) => "r",
            KeyType::Table(_) => "t",
            KeyType::Queue(_) => "q",
        };
    }
}

impl Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Table(key) => ("t", key),
            Self::Record(key) => ("r", key),
            Self::Queue(key) => ("q", key),
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

impl<'a> PartialEq for KeyType {
    fn eq(&self, other: &Self) -> bool {
        if (self.is_record() && other.is_record()) || (self.is_table() && other.is_table() || (self.is_queue() && other.is_queue())) {
            if self.get_key() == other.get_key() {
                return true;
            }
        }
        return false;
    }
}

///
/// Type of the value
///
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// This is a table pointer, belongs to `KeyType::Table`
    TablePointer(super::Table),

    /// This is a record pointer, belongs to `KeyType::Record`
    RecordPointer(String),

    /// This is a queue pointer, belongs to `KeyType::Queue`
    QueuePointer(VecDeque<String>),
}

impl ValueType {
    /// Tells that it is a `ValueType::TablePointer`
    pub fn is_table(&self) -> bool {
        return match self {
            ValueType::TablePointer(_) => true,
            _ => false,
        };
    }

    /// Tells that it is a `ValueType::RecordPointer`
    pub fn is_record(&self) -> bool {
        return match self {
            ValueType::RecordPointer(_) => true,
            _ => false,
        };
    }

    /// Tells that it is a `ValueType::QueuePointer`
    pub fn is_queue(&self) -> bool {
        return match self {
            ValueType::QueuePointer(_) => true,
            _ => false,
        };
    }

    pub fn get_value(&self) -> &str {
        return match self {
            Self::TablePointer(_) => "TablePointer",
            Self::RecordPointer(key) => &key[..],
            Self::QueuePointer(_) => "QueuePointer",
        };
    }
}
