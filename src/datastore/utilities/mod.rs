//! Built-in utilities

use std::{
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

pub(crate) mod internal;

use crate::hook::{
    enums::{HookManagerAction, HookManagerResponse},
    utilities::get_channel,
};

use super::{
    enums::{error::ErrorKind, pair::KeyType, pair::ValueType, DatabaseAction, ListType},
    types::{ResultWithList, ResultWithResult, ResultWithoutResult, Table, ResultWithHook, ResultWithHooks},
    Database,
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
/// let (sender, _) = start_datastore("root".to_string(), None);
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
pub fn start_datastore(
    name: String,
    sender: Option<Sender<HookManagerAction>>,
) -> (Sender<DatabaseAction>, JoinHandle<()>) {
    let (tx, rx) = std::sync::mpsc::channel::<DatabaseAction>();

    let thread = std::thread::spawn(move || {
        let mut db = Database::new(name).expect("Failed to allocate database");

        if let Some(sender) = sender {
            db.subscribe_to_hook_manager(sender);
        }

        while let Ok(data) = rx.recv() {
            match data {
                // Handle Get actions
                DatabaseAction::Get(sender, key) => match db.get(KeyType::Record(key)) {
                    Ok(value) => send_response!(sender, Ok(value)),
                    Err(e) => send_response!(sender, Err(e)),
                },
                // Handle Set actions
                DatabaseAction::Set(sender, key, value) => {
                    match db.insert(KeyType::Record(key), ValueType::RecordPointer(value)) {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }
                }
                // Handle DeleteKey actions
                DatabaseAction::DeleteKey(sender, key) => {
                    match db.delete_key(KeyType::Record(key)) {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }
                }
                // Handle DeleteTable actions
                DatabaseAction::DeleteTable(sender, key) => {
                    match db.delete_table(KeyType::Table(key)) {
                        Ok(_) => send_response!(sender, Ok(())),
                        Err(e) => send_response!(sender, Err(e)),
                    }
                }
                // Handle ListKeys action
                DatabaseAction::ListKeys(sender, key, level) => {
                    match db.list_keys(KeyType::Record(key), level) {
                        Ok(list) => send_response!(sender, Ok(list)),
                        Err(e) => send_response!(sender, Err(e)),
                    }
                }
                // Set hook
                DatabaseAction::HookSet(sender, prefix, link) => match &db.hook_sender {
                    Some(hook_sender) => {
                        let (tx, rx) = get_channel();
                        let action = HookManagerAction::Set(tx, prefix, link);
                        hook_send!(sender, hook_sender, action);

                        match rx.recv() {
                            Ok(response) => match response {
                                HookManagerResponse::Ok => send_response!(sender, Ok(())),
                                _ => send_response!(
                                    sender,
                                    Err(ErrorKind::InternalError("Failed to add hook".to_string()))
                                ),
                            },
                            Err(e) => hook_receive_failed!(sender, e),
                        }
                    }
                    None => hook_inactive!(sender),
                },
                // Get links for specific hook
                DatabaseAction::HookGet(sender, prefix) => match &db.hook_sender {
                    Some(hook_sender) => {
                        let (tx, rx) = get_channel();
                        let action = HookManagerAction::Get(tx, prefix);
                        hook_send!(sender, hook_sender, action);

                        match rx.recv() {
                            Ok(response) => match response {
                                HookManagerResponse::Hook(prefix, hooks) => {
                                    send_response!(sender, Ok((prefix, hooks)))
                                }
                                _ => send_response!(
                                    sender,
                                    Err(ErrorKind::InvalidKey("Hook is not found".to_string()))
                                ),
                            },
                            Err(e) => hook_receive_failed!(sender, e),
                        }
                    }
                    None => hook_inactive!(sender),
                },
                // List hooks
                DatabaseAction::HookList(sender, prefix) => match &db.hook_sender {
                    Some(hook_sender) => {
                        let (tx, rx) = get_channel();
                        let action = HookManagerAction::List(tx, prefix);

                        hook_send!(sender, hook_sender, action);

                        match rx.recv() {
                            Ok(response) => match response {
                                HookManagerResponse::HookList(list) => {
                                    send_response!(sender, Ok(list))
                                }
                                _ => send_response!(
                                    sender,
                                    Err(ErrorKind::InvalidKey("Hook is not found".to_string()))
                                ),
                            },
                            Err(e) => hook_receive_failed!(sender, e),
                        }
                    }
                    None => hook_inactive!(sender),
                },
                // Remove existing hooks
                DatabaseAction::HookRemove(sender, prefix, link) => match &db.hook_sender {
                    Some(hook_sender) => {
                        let (tx, rx) = get_channel();
                        let action = HookManagerAction::Remove(tx, prefix, link);

                        hook_send!(sender, hook_sender, action);

                        match rx.recv() {
                            Ok(response) => match response {
                                HookManagerResponse::Ok => send_response!(sender, Ok(())),
                                _ => send_response!(
                                    sender,
                                    Err(ErrorKind::InvalidKey("Hook is not found".to_string()))
                                ),
                            },
                            Err(e) => hook_receive_failed!(sender, e),
                        }
                    }
                    None => hook_inactive!(sender),
                },
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

/// Return with channel for HookSet action
pub fn get_channel_for_hook_set() -> (Sender<ResultWithoutResult>, Receiver<ResultWithoutResult>) {
    return std::sync::mpsc::channel::<ResultWithoutResult>();
}

/// Return with channel for HookGet action
pub fn get_channel_for_hook_get() -> (Sender<ResultWithHook>, Receiver<ResultWithHook>) {
    return std::sync::mpsc::channel::<ResultWithHook>();
}

/// Return with channel for HookRemove action
pub fn get_channel_for_hook_remove() -> (Sender<ResultWithoutResult>, Receiver<ResultWithoutResult>) {
    return std::sync::mpsc::channel::<ResultWithoutResult>();
}

/// Return with channel for HookList action
pub fn get_channel_for_hook_list() -> (Sender<ResultWithHooks>, Receiver<ResultWithHooks>) {
    return std::sync::mpsc::channel::<ResultWithHooks>();
}

macro_rules! hook_inactive {
    ($sender:expr) => {
        $sender
            .send(Err(ErrorKind::InactiveHookManager))
            .unwrap_or_else(|e| eprintln!("Error during send: {}", e))
    };
}
pub(self) use hook_inactive;

macro_rules! hook_send {
    ($sender:expr, $hook_sender:expr, $action:expr) => {
        if let Err(e) = $hook_sender.send($action) {
            eprintln!("Failed to send to hook manager: {}", e);
            $sender
                .send(Err(ErrorKind::InternalError("".to_string())))
                .unwrap_or_else(|e| eprintln!("Error during send: {}", e));
            continue;
        }
    };
}
pub(self) use hook_send;

macro_rules! hook_receive_failed {
    ($sender:expr, $error:expr) => {{
        eprintln!("Failed to receive from hook manager: {}", $error);
        $sender
            .send(Err(ErrorKind::InternalError(
                "Failed to receive from hook manager".to_string(),
            )))
            .unwrap_or_else(|e| eprintln!("Error during send: {}", e));
    }};
}
pub(self) use hook_receive_failed;

macro_rules! send_response {
    ($sender:expr, $value:expr) => {{
        $sender
            .send($value)
            .unwrap_or_else(|e| eprintln!("Error during send: {}", e));
    }};
}
pub(self) use send_response;
