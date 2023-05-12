#[cfg(test)]
mod tests {
    use crate::{
        controller::Database,
        enums::{KeyType, ListType, ValueType},
    };

    #[test]
    fn basic_functions() {
        let db = Database::new("root".to_string());
        assert_eq!(true, db.is_ok());

        let mut db = db.unwrap();

        // Insert some data
        let response = db.insert(
            KeyType::Record("/root/status".to_string()),
            ValueType::RecordPointer("okay".to_string()),
        );
        assert_eq!(true, response.is_ok());

        let response = db.insert(
            KeyType::Record("/root/status/sub1".to_string()),
            ValueType::RecordPointer("okay".to_string()),
        );
        assert_eq!(true, response.is_ok());

        let response = db.insert(
            KeyType::Record("/root/status/sub2".to_string()),
            ValueType::RecordPointer("okay".to_string()),
        );
        assert_eq!(true, response.is_ok());

        let response = db.insert(
            KeyType::Record("/root/node_name".to_string()),
            ValueType::RecordPointer("teszt1".to_string()),
        );
        assert_eq!(true, response.is_ok());

        let response = db.insert(
            KeyType::Record("/root/network/gitea".to_string()),
            ValueType::RecordPointer("okay".to_string()),
        );
        assert_eq!(true, response.is_ok());

        // Check that value has been saved
        let value = db.get(KeyType::Record("/root/status".to_string()));
        assert_eq!(true, value.is_ok());

        let value = match value.unwrap() {
            ValueType::RecordPointer(value) => value,
            _ => panic!(),
        };
        assert_eq!("okay".to_string(), *value);

        // Get non exist key
        let response = db.get(KeyType::Record("/root/asd/eqq".to_string()));
        assert_eq!(true, response.is_err());

        // Check override value
        let response = db.insert(
            KeyType::Record("/root/status".to_string()),
            ValueType::RecordPointer("great".to_string()),
        );
        assert_eq!(true, response.is_ok());

        match db.get(KeyType::Record("/root/status".to_string())) {
            Ok(value) => match value {
                ValueType::RecordPointer(text) => assert_eq!("great".to_string(), *text),
                _ => panic!("It should be record pointer"),
            },
            Err(e) => panic!("{}", e),
        }

        // Check some error
        let response = db.insert(
            KeyType::Record("/status".to_string()),
            ValueType::RecordPointer("okay".to_string()),
        );
        assert_eq!(true, response.is_err());

        let response = db.insert(
            KeyType::Record("root/batch/error/plan1".to_string()),
            ValueType::RecordPointer("failed".to_string()),
        );
        assert_eq!(true, response.is_err());

        // Check listing
        match db.list_keys(KeyType::Record("/root".to_string()), ListType::All) {
            Ok(table) => {
                assert_eq!(true, table.len() >= 1);
            }
            Err(e) => panic!("{}", e),
        }

        match db.list_keys(KeyType::Record("/root/network".to_string()), ListType::All) {
            Ok(table) => {
                assert_eq!(true, table.len() >= 1);
            }
            Err(e) => panic!("{}", e),
        }

        match db.list_keys(KeyType::Record("/root".to_string()), ListType::OneLevel) {
            Ok(table) => {
                assert_eq!(true, table.len() >= 1);
            }
            Err(e) => panic!("{}", e),
        }

        // Try to list non-exist route
        let a = db.list_keys(KeyType::Record("/root/asd/eqq".to_string()), ListType::All);
        assert_eq!(true, a.is_err());

        // Delete key
        let response = db.delete_key(KeyType::Record("/root/status".to_string()));
        assert_eq!(true, response.is_ok());

        let response = db.get(KeyType::Record("/root/status".to_string()));
        assert_eq!(true, response.is_err());

        let response = db.delete_key(KeyType::Record("/root/status".to_string()));
        assert_eq!(true, response.is_err());

        // Drop table
        let response = db.delete_table(KeyType::Table("/root/status".to_string()));
        assert_eq!(true, response.is_ok());

        let response = db.get(KeyType::Record("/root/status/sub1".to_string()));
        assert_eq!(true, response.is_err());

    }
}
