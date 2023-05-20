//! Built-in utilities

use std::sync::mpsc::{Receiver, Sender};

use super::{
    Database,
    enums::{DatabaseAction, error::ErrorKind, pair::KeyType, ListType, pair::ValueType},
    types::{ResultWithList, ResultWithResult, ResultWithoutResult, Table},
};

/// Initialize database on another thread, create a channel and return with it
///
/// # Example for call
///
/// ```
/// use onlyati_datastore::datastore::{
///     enums::{error::ErrorKind, DatabaseAction, pair::ValueType},
///     utilities::{start_datastore, self},
/// };
///
/// let sender = start_datastore("root".to_string());
///
/// // Add a new pair
/// let (tx, rx) = utilities::get_channel_for_set();
/// let set_action = DatabaseAction::Set(tx, "/root/network".to_string(), "ok".to_string());
///
/// sender.send(set_action).expect("Failed to send the request");
/// rx.recv().unwrap();
///
/// // Get the pair
/// let (tx, rx) = utilities::get_channel_for_get();
/// let get_action = DatabaseAction::Get(tx, "/root/network".to_string());
///
/// sender.send(get_action).expect("Failed to send the get request");
/// let data = rx.recv().expect("Failed to receive message").expect("Failed to get data");
/// assert_eq!(ValueType::RecordPointer("ok".to_string()), data);
/// ```
pub fn start_datastore(name: String) -> std::sync::mpsc::Sender<DatabaseAction> {
    let (tx, rx) = std::sync::mpsc::channel::<DatabaseAction>();

    std::thread::spawn(move || {
        let mut db = Database::new(name).expect("Failed to allocate database");

        while let Ok(data) = rx.recv() {
            match data {
                // Handle Get actions
                DatabaseAction::Get(sender, key) => match db.get(KeyType::Record(key)) {
                    Ok(value) => sender
                        .send(Ok(value))
                        .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                    Err(e) => sender
                        .send(Err(e))
                        .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                },
                // Handle Set actions
                DatabaseAction::Set(sender, key, value) => {
                    match db.insert(KeyType::Record(key), ValueType::RecordPointer(value)) {
                        Ok(_) => sender
                            .send(Ok(()))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                        Err(e) => sender
                            .send(Err(e))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                    }
                }
                // Handle DeleteKey actions
                DatabaseAction::DeleteKey(sender, key) => {
                    match db.delete_key(KeyType::Record(key)) {
                        Ok(_) => sender
                            .send(Ok(()))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                        Err(e) => sender
                            .send(Err(e))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                    }
                }
                // Handle DeleteTable actions
                DatabaseAction::DeleteTable(sender, key) => {
                    match db.delete_table(KeyType::Table(key)) {
                        Ok(_) => sender
                            .send(Ok(()))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                        Err(e) => sender
                            .send(Err(e))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                    }
                }
                // Handle ListKeys action
                DatabaseAction::ListKeys(sender, key, level) => {
                    match db.list_keys(KeyType::Record(key), level) {
                        Ok(list) => sender
                            .send(Ok(list))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                        Err(e) => sender
                            .send(Err(e))
                            .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                    }
                }
            }
        }
    });

    return tx;
}

/// Return with channel for Set action
pub fn get_channel_for_set() -> (Sender<ResultWithoutResult>, Receiver<ResultWithoutResult>) {
    return std::sync::mpsc::channel::<ResultWithoutResult>();
}

/// Return with channel for Get action
pub fn get_channel_for_get() -> (Sender<ResultWithResult>, Receiver<ResultWithResult>) {
    return std::sync::mpsc::channel::<ResultWithResult>();
}

/// Return with channel for DeleteKey and DeleteTable actions
pub fn get_channel_for_delete() -> (Sender<ResultWithoutResult>, Receiver<ResultWithoutResult>) {
    return std::sync::mpsc::channel::<ResultWithoutResult>();
}

/// Return with channel for ListKeys action
pub fn get_channel_for_list() -> (Sender<ResultWithList>, Receiver<ResultWithList>) {
    return std::sync::mpsc::channel::<ResultWithList>();
}

/// Validate and parse the key string.
/// For example: /root/status/sub1 -> ["root", "status", "sub1"]
pub(crate) fn validate_key<'a>(
    key_string: &'a String,
    db_name: &String,
) -> Result<Vec<&'a str>, ErrorKind> {
    if &key_string[0..1] != "/" {
        return Err(ErrorKind::InvalidKey(
            "Key must begin with '/' sign".to_string(),
        ));
    }

    let key_routes = key_string
        .split("/")
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>();

    if key_routes.len() < 1 {
        return Err(ErrorKind::InvalidKey(
            "Key must contain at least 1 items, e.g.: /root/status".to_string(),
        ));
    }

    if key_routes[0] != db_name {
        return Err(ErrorKind::InvalidKey(
            "Key does not begin with the root table".to_string(),
        ));
    }

    return Ok(key_routes);
}

/// Recursive algoritm to find a table
pub(crate) fn find_table<'a>(db: Box<&'a Table>, routes: Vec<&'a str>) -> Option<Box<&'a Table>> {
    if routes.len() == 0 {
        return Some(db);
    }

    let current_table = KeyType::Table(routes[0].to_string());
    match db.get(&current_table) {
        Some(table) => match table {
            ValueType::TablePointer(table_pointer) => {
                return find_table(Box::new(table_pointer), routes[1..].to_vec());
            }
            _ => return None,
        },
        _ => return None,
    }
}

/// Recursive algoritm the find a table and return as mutable reference
pub(crate) fn find_table_mut<'a>(
    db: Box<&'a mut Table>,
    routes: Vec<&'a str>,
) -> Option<Box<&'a mut Table>> {
    if routes.len() == 0 {
        return Some(db);
    }

    let current_table = KeyType::Table(routes[0].to_string());
    match db.get_mut(&current_table) {
        Some(table) => match table {
            ValueType::TablePointer(table_pointer) => {
                return find_table_mut(Box::new(table_pointer), routes[1..].to_vec());
            }
            _ => return None,
        },
        _ => return None,
    }
}

/// Display all items from a table
pub(crate) fn display_tables(
    db: Box<&Table>,
    key_prefix: &String,
    level: &ListType,
) -> Result<Vec<KeyType>, ErrorKind> {
    let mut result: Vec<KeyType> = Vec::with_capacity(std::mem::size_of::<KeyType>() * db.len());

    for (key, value) in db.iter() {
        match key {
            KeyType::Record(key) => {
                let new_key = KeyType::Record(key_prefix.clone() + "/" + key);
                result.push(new_key);
            }
            KeyType::Table(key) => {
                if *level == ListType::OneLevel {
                    continue;
                }

                let table_name = match value {
                    ValueType::TablePointer(table) => table,
                    _ => continue,
                };
                let mut temp = display_tables(
                    Box::new(table_name),
                    &format!("{}/{}", key_prefix, key),
                    level,
                )?;

                result.append(&mut temp);
            }
        }
    }

    return Ok(result);
}
