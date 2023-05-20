//! Built-in utilities

use std::{sync::mpsc::{Receiver, Sender}, thread::JoinHandle};

pub(crate) mod inernal;

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
pub fn start_datastore(name: String) -> (Sender<DatabaseAction>, JoinHandle<()>) {
    let (tx, rx) = std::sync::mpsc::channel::<DatabaseAction>();

    let thread = std::thread::spawn(move || {
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

    return (tx, thread);
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
