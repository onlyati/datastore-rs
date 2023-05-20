use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::JoinHandle;

use super::enums::{HookManagerAction, HookManagerResponse};
use super::HookManager;

/// Start a HookManager on a single tokio thread
/// 
/// # Examples
/// ```
/// use onlyati_datastore::hook::utilities;
/// use onlyati_datastore::hook::enums::{HookManagerAction, HookManagerResponse};
/// 
/// let (sender, _) = utilities::start_hook_manager();
/// 
/// let (tx, rx) = utilities::get_channel();
/// let action = HookManagerAction::Set(tx, "/root/stats".to_string(), "http://127.0.0.1:3031".to_string());
/// 
/// sender.send(action).expect("Failed to send request");
/// 
/// let response = rx.recv().expect("Failed to receive");
/// assert_eq!(HookManagerResponse::Ok, response);
/// 
/// ```
pub fn start_hook_manager() -> (Sender<HookManagerAction>, JoinHandle<()>) {
    let (tx, rx) = channel::<HookManagerAction>();
    let mut manager = HookManager::new();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to allocate runtime for HookManager");

    let thread = std::thread::spawn(move || {
        rt.block_on(async move {
            loop {
                match rx.recv() {
                    Ok(request) => match request {
                        HookManagerAction::Set(sender, prefix, target) => {
                            match manager.add(prefix, target) {
                                Ok(_) => sender
                                    .send(HookManagerResponse::Ok)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                                Err(e) => sender
                                    .send(e)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                            }
                        }
                        HookManagerAction::Remove(sender, prefix, target) => {
                            match manager.remove(prefix, target) {
                                Ok(_) => sender
                                    .send(HookManagerResponse::Ok)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                                Err(e) => sender
                                    .send(e)
                                    .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                            }
                        }
                        HookManagerAction::Get(sender, prefix) => match manager.get(&prefix) {
                            Some(hooks) => sender
                                .send(HookManagerResponse::Hook(prefix, hooks))
                                .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                            None => sender
                                .send(HookManagerResponse::Error("Not found".to_string()))
                                .unwrap_or_else(|e| eprintln!("Error during send: {}", e)),
                        },
                        HookManagerAction::List(sender, prefix) => {
                            sender
                                .send(HookManagerResponse::HookList(manager.list(&prefix)))
                                .unwrap_or_else(|e| eprintln!("Error during send: {}", e));
                        }
                        HookManagerAction::Send(test_key, value) => {
                            manager.execute_hooks(&test_key, &value).await;
                        }
                    },
                    Err(e) => panic!("Hook manager failed: {}", e),
                }
            }
        });
    });

    return (tx, thread);
}

/// Get channel for HookManager response
pub fn get_channel() -> (Sender<HookManagerResponse>, Receiver<HookManagerResponse>) {
    return channel::<HookManagerResponse>();
}