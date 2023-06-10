use super::{
    Table, {ErrorKind, KeyType, ListType, ValueType},
};

/// Validate and parse the key string.
/// For example: /root/status/sub1 -> ["root", "status", "sub1"]
pub(crate) fn validate_key<'a>(
    key_string: &'a str,
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
pub(crate) fn display_tables<'a>(
    db: Box<&Table>,
    key_prefix: &String,
    level: &ListType,
) -> Result<Vec<KeyType>, ErrorKind> {
    let mut result: Vec<KeyType> = Vec::with_capacity(std::mem::size_of::<KeyType>() * db.len());

    for (key, value) in db.iter() {
        match key {
            KeyType::Record(key) => {
                let new_key = format!("{}/{}", key_prefix.clone(), key);
                let new_key = KeyType::Record(new_key);
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
            KeyType::Queue(key) => {
                let new_key = format!("{}/{}", key_prefix.clone(), key);
                let new_key = KeyType::Queue(new_key);
                result.push(new_key);
            }
        }
    }

    return Ok(result);
}
