//! Main component

pub mod enums;
pub mod types;
pub mod utilities;

use self::{
    enums::{error::ErrorKind, pair::KeyType, ListType, pair::ValueType},
    types::Table,
};

/// Database struct
pub struct Database {
    /// Name of database
    name: String,

    /// Pointer to the root table
    root: Table,
}

impl Database {
    /// Create new database and return with the struct.
    ///
    /// # Arguments
    /// 1. `root_name` - Name of database
    ///
    /// # Examples
    /// ```
    /// let db = onlyati_datastore::datastore::Database::new("root".to_string()).unwrap();
    /// ```
    pub fn new(root_name: String) -> Result<Self, ErrorKind> {
        if root_name.contains("/") {
            return Err(ErrorKind::InvalidRoot(
                "Root name cannot contains '/' character".to_string(),
            ));
        }

        return Ok(Self {
            name: root_name,
            root: Table::new(),
        });
    }

    /// Insert or update key into database. Return with nothing if the insert was successful. Else with an error code.
    ///
    /// # Arguments
    /// 1. `key` - Unique key for data
    /// 1. `value` - Value that is assigned for the key
    ///
    /// # Example
    ///
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    ///
    /// let result = db.insert(KeyType::Record("/root/network/dns-stats".to_string()), ValueType::RecordPointer("ok".to_string()));
    /// ```
    pub fn insert(&mut self, key: KeyType, value: ValueType) -> Result<(), ErrorKind> {
        let key_routes = utilities::validate_key(key.get_key(), &self.name)?;

        let mut table = Box::new(&mut self.root);
        let last_route = key_routes[key_routes.len() - 1];
        let mut route_index: usize = 0;
        let mut current_route = key_routes[route_index].to_string();

        while last_route != current_route {
            let temp_key = KeyType::Table(current_route.clone());
            table
                .entry(temp_key.clone())
                .or_insert(ValueType::TablePointer(Table::new()));

            *table = match table.get_mut(&temp_key) {
                Some(item) => match item {
                    ValueType::TablePointer(sub_table) => sub_table,
                    _ => {
                        return Err(ErrorKind::InternalError(
                            "This should not have happen".to_string(),
                        ))
                    }
                },
                _ => {
                    return Err(ErrorKind::InternalError(
                        "This should not have happen".to_string(),
                    ))
                }
            };

            route_index += 1;
            current_route = key_routes[route_index].to_string();
        }

        let record_key = KeyType::Record(last_route.to_string());
        table.insert(record_key, value);

        return Ok(());
    }

    /// Get the value of a key and return with a copy of it. If not found return with error.
    ///
    /// # Arguments
    /// 1. `key` - Unique key that has to be found
    /// 
    /// # Example
    /// 
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    /// 
    /// db.insert(KeyType::Record("/root/status".to_string()), ValueType::RecordPointer("Having a great time".to_string())).expect("Failed to insert");
    /// let value = db.get(KeyType::Record("/root/status".to_string())).expect("Key not found");
    /// ```
    pub fn get(&self, key: KeyType) -> Result<ValueType, ErrorKind> {
        if let KeyType::Table(_) = key {
            return Err(ErrorKind::InvalidKey(
                "Parameter must be a Record type".to_string(),
            ));
        }

        let key_routes = utilities::validate_key(key.get_key(), &self.name)?;
        let table = match utilities::find_table(
            Box::new(&self.root),
            key_routes[..key_routes.len() - 1].to_vec(),
        ) {
            Some(table) => table,
            None => {
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        };

        let find_key = KeyType::Record(key_routes[key_routes.len() - 1].to_string());

        match table.get(&find_key) {
            Some(value) => return Ok(value.clone()),
            None => {
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        }
    }

    /// List keys from a specific entry point and return with a key list. If failed return with error.
    /// 
    /// # Arguments
    /// 1. `key_prefix` - Path where the keys has to be collected
    /// 1. `level` - Need all inner level (`ListType::All`) or just current level (`ListType::OneLevel`)
    /// 
    /// # Example
    /// 
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::{pair::KeyType, pair::ValueType, ListType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    /// 
    /// db.insert(KeyType::Record("/root/status/sub1".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub2".to_string()), ValueType::RecordPointer("PING NOK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub3".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// let list = db.list_keys(KeyType::Record("/root/status".to_string()), ListType::All).expect("Key not found");
    /// 
    /// println!("{:?}", list);
    /// ```
    pub fn list_keys(
        &self,
        key_prefix: KeyType,
        level: ListType,
    ) -> Result<Vec<KeyType>, ErrorKind> {
        if let KeyType::Table(_) = key_prefix {
            return Err(ErrorKind::InvalidKey(
                "Parameter must be a Record type".to_string(),
            ));
        }

        // Find the base table
        let key_routes = utilities::validate_key(key_prefix.get_key(), &self.name)?;
        let table = match utilities::find_table(Box::new(&self.root), key_routes) {
            Some(table) => table,
            None => {
                return Err(ErrorKind::InvalidKey(
                    "Specified route does not exist".to_string(),
                ))
            }
        };

        // Get the information
        let result = utilities::display_tables(table, key_prefix.get_key(), &level)?;

        return Ok(result);
    }

    /// Delete specific key, return with nothig if successful, else with error message.
    /// 
    /// # Arguments
    /// 1. `key` - Unique key that has to be deleted
    /// 
    /// # Example
    /// 
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::pair::{KeyType, ValueType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    /// 
    /// let key = KeyType::Record("/root/status".to_string());
    /// db.insert(key.clone(), ValueType::RecordPointer("Having a great time".to_string())).expect("Failed to insert");
    /// db.delete_key(key).expect("Could not delete the key");
    /// ```
    pub fn delete_key(&mut self, key: KeyType) -> Result<(), ErrorKind> {
        if let KeyType::Table(_) = key {
            return Err(ErrorKind::InvalidKey(
                "Parameter must be a Record type".to_string(),
            ));
        }

        let key_routes = utilities::validate_key(key.get_key(), &self.name)?;
        let table = match utilities::find_table_mut(
            Box::new(&mut self.root),
            key_routes[..key_routes.len() - 1].to_vec(),
        ) {
            Some(table) => table,
            None => {
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        };

        let delete_key = KeyType::Record(key_routes[key_routes.len() - 1].to_string());

        match table.remove(&delete_key) {
            Some(_) => return Ok(()),
            None => {
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        };
    }

    /// Drop the whole table. If successful return with nothing else with error message.
    /// 
    /// # Arguments
    /// 1. `key` - Key that which table has to be deleted
    /// 
    /// # Example
    /// 
    /// ```
    /// use onlyati_datastore::datastore::Database;
    /// use onlyati_datastore::datastore::enums::{pair::KeyType, pair::ValueType, ListType};
    ///
    /// let mut db = Database::new("root".to_string()).unwrap();
    /// 
    /// db.insert(KeyType::Record("/root/status/sub1".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub2".to_string()), ValueType::RecordPointer("PING NOK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/status/sub3".to_string()), ValueType::RecordPointer("PING OK".to_string())).expect("Failed to insert");
    /// db.insert(KeyType::Record("/root/node_name".to_string()), ValueType::RecordPointer("vps01".to_string())).expect("Failed to insert");
    /// 
    /// db.delete_table(KeyType::Table("/root/status".to_string())).expect("Failed to drop from status table");
    /// 
    /// // Only "node_name" remain in the list
    /// let list = db.list_keys(KeyType::Record("/root".to_string()), ListType::All).expect("Key not found");
    /// println!("{:?}", list);
    /// ```
    pub fn delete_table(&mut self, key: KeyType) -> Result<(), ErrorKind> {
        if let KeyType::Record(_) = key {
            return Err(ErrorKind::InvalidKey(
                "Parameter must be a Table type".to_string(),
            ));
        }

        let key_routes = utilities::validate_key(key.get_key(), &self.name)?;
        let table = match utilities::find_table_mut(
            Box::new(&mut self.root),
            key_routes[..key_routes.len() - 1].to_vec(),
        ) {
            Some(table) => table,
            None => {
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        };

        let delete_key = KeyType::Table(key_routes[key_routes.len() - 1].to_string());

        match table.remove(&delete_key) {
            Some(_) => return Ok(()),
            None => {
                return Err(ErrorKind::InvalidKey(
                    "Specified key does not exist".to_string(),
                ))
            }
        };
    }
}
