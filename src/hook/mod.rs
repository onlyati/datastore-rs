//! Main component

use std::collections::{BTreeMap, HashMap};

pub mod enums;
pub mod types;
pub mod utilities;

use enums::HookManagerResponse;
use types::{Hooks, Prefix};

/// HookManager main structure
pub struct HookManager {
    hooks: BTreeMap<Prefix, Hooks>,
}

impl HookManager {
    /// Allocate new HookManager
    pub fn new() -> Self {
        return HookManager {
            hooks: BTreeMap::new(),
        };
    }

    /// Add new hook
    pub fn add(&mut self, prefix: String, link: String) -> Result<(), HookManagerResponse> {
        match self.hooks.get_mut(&prefix) {
            Some(hooks) => match hooks.iter().position(|x| x == &link) {
                Some(_) => return Err(HookManagerResponse::Error("Already defined".to_string())),
                None => {
                    hooks.push(link);
                    return Ok(());
                }
            },
            None => {
                self.hooks.insert(prefix, vec![link]);
                return Ok(());
            }
        }
    }

    /// Delete existing hook
    pub fn remove(&mut self, prefix: String, link: String) -> Result<(), HookManagerResponse> {
        match self.hooks.get_mut(&prefix) {
            Some(hooks) => match hooks.iter().position(|x| x == &link) {
                Some(index) => {
                    hooks.remove(index);
                    return Ok(());
                }
                None => return Err(HookManagerResponse::Error("Not found".to_string())),
            },
            None => return Err(HookManagerResponse::Error("Not found".to_string())),
        }
    }

    /// Check that hook exist
    pub fn get(&self, prefix: &String) -> Option<Hooks> {
        match self.hooks.get(prefix) {
            Some(hooks) => return Some(hooks.clone()),
            None => return None,
        }
    }

    /// List hooks for specified paths
    pub fn list(&self, key: &String) -> BTreeMap<Prefix, Hooks> {
        let selected_hooks: BTreeMap<Prefix, Hooks> = self
            .hooks
            .iter()
            .filter(|x| x.0.starts_with(key))
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        return selected_hooks;
    }

    /// Pass a key and send POST request if key match with any defined prefix
    pub async fn execute_hooks(&self, key: &String, value: &String) -> Option<i32> {
        let client = reqwest::Client::new();
        let mut body = HashMap::new();
        body.insert("key", key);
        body.insert("value", value);

        let mut counter = 0;

        for (prefix, links) in &self.hooks {
            if key.starts_with(prefix) {
                for link in links {
                    counter += 1;
                    match client.post(link).json(&body).send().await {
                        Err(e) => eprintln!("Error: HTTP request with hook but: {}", e),
                        _ => (),
                    };
                }
            }
        }

        match counter {
            0 => return None,
            i => return Some(i),
        }
    }
}
