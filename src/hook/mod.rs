//! Main component

use std::collections::{BTreeMap, HashMap};

pub mod enums;
pub mod types;
pub mod utilities;

use enums::HookManagerResponse;
use types::{Hooks, Prefix};

/// HookManager main structure
/// 
/// # Examples
/// ```
/// use onlyati_datastore::hook::HookManager;
/// 
/// let mut manager = HookManager::new();
/// 
/// let mut manager = HookManager::new();
/// 
/// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3031".to_string());
/// assert_eq!(true, result.is_ok());
/// 
/// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3032".to_string());
/// assert_eq!(true, result.is_ok());
/// 
/// let result = manager.add("/root/arpa".to_string(), "http://127.0.0.1:3031".to_string());
/// assert_eq!(true, result.is_ok());
/// 
/// let result = manager.list(&"/root".to_string());
/// assert_eq!(2, result.len());
/// 
/// let result = manager.list(&"/root/stat".to_string());
/// assert_eq!(1, result.len());
/// 
/// let result = manager.list(&"/root/no_exist".to_string());
/// assert_eq!(0, result.len());
/// ```
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
    /// 
    /// # Examples
    /// ```
    /// use onlyati_datastore::hook::HookManager;
    /// 
    /// let mut manager = HookManager::new();
    /// 
    /// let mut manager = HookManager::new();
    /// 
    /// // Normaly you have to specify address where the HTTP POST request can be sent
    /// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3031".to_string());
    /// assert_eq!(true, result.is_ok());
    /// 
    /// let result = manager.add("/root/status".to_string(), "http://127.0.0.1:3032".to_string());
    /// assert_eq!(true, result.is_ok());
    /// 
    /// let rt = tokio::runtime::Builder::new_current_thread()
    ///     .enable_all()
    ///     .build()
    ///     .unwrap();
    /// rt.block_on(async move {
    ///     let counter = manager.execute_hooks(&"/root/status/dns1".to_string(), &"okay".to_string()).await;
    ///     assert_eq!(Some(2), counter);
    /// 
    ///     let counter = manager.execute_hooks(&"/root/no_exist".to_string(), &"okay".to_string()).await;
    ///     assert_eq!(None, counter);
    /// });
    /// 
    /// ```
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
