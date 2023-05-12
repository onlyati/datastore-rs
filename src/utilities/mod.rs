//! Built-in utilities

mod tests;

use crate::{
    controller::Database,
    enums::{DatabaseAction, ErrorKind, KeyType, ListType, ValueType},
    types::Table,
};

/// Initialize database on another thread, create a channel and return with it
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
