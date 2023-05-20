use std::sync::mpsc::{channel, Sender};

use super::enums::{HookManagerAction, HookManagerResponse};
use super::HookManager;

pub fn start_hook_manager() -> Sender<HookManagerAction> {
    let (tx, rx) = channel::<HookManagerAction>();
    let mut manager = HookManager::new();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to allocate runtime for HookManager");

    std::thread::spawn(move || {
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
                        HookManagerAction::Send(sender, test_key, value) => {
                            manager.execute_hooks(&test_key, &value).await;
                            sender
                                .send(HookManagerResponse::Ok)
                                .unwrap_or_else(|e| eprintln!("Error during send: {}", e));
                        }
                    },
                    Err(e) => panic!("Hook manager failed: {}", e),
                }
            }
        });
    });

    return tx;
}
