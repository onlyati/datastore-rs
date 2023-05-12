# OnlyAti.Datastore
 
This a very simple key-value data store. Pairs are stored tables according their key name.
Each key is split for more routes at '/' character and this route lead to the place of key.

Simple visual representaion with the following keys:
- /root/status/sub1
- /root/status/sub2
- /root/network/dns
- /root/network/www
```plain
+---------+
| status  | ---------> +------+
+---------+            | sub1 |
| network | ------+     +------+
+---------+       |    | sub2 |
   root           |    +------+
                  |     status
                  |
                  +--> +-----+
                       | dns |
                       +-----+
                       | www |
                       +-----+
```

# Sample code to use the built-in thread server

There is a provided function that creates a thread, initialize database then return with a `std::sync::mpsc::Sender` so other thread can send request.
This a simple method to initialize this database, communication can be done by using channels.

```rust
use onlyati_datastore::{
    enums::{ErrorKind, DatabaseAction, ValueType},
    utilities::start_datastore,
};

let sender = start_datastore("root".to_string());

// Add a new pair
let (tx, rx) = std::sync::mpsc::channel::<Result<(), ErrorKind>>();
let set_action = DatabaseAction::Set(tx, "/root/network".to_string(), "ok".to_string());

sender.send(set_action).expect("Failed to send the request");
rx.recv().unwrap(); 

// Get the pair
let (tx, rx) = std::sync::mpsc::channel::<Result<ValueType, ErrorKind>>();
let get_action = DatabaseAction::Get(tx, "/root/network".to_string());

sender.send(get_action).expect("Failed to send the get request");
let data = rx.recv().expect("Failed to receive message").expect("Failed to get data");
assert_eq!(ValueType::RecordPointer("ok".to_string()), data);
```


# Sample code to run without built-in thread

There is a provided function that created a thread, initialize database then return with a `std::sync::mpsc::Sender` so other thread can send request.
But it is also possible to use it as it is called directly if the application does not prefer the method mentioned earlier.

```rust
use onlyati_datastore::controller::Database;
use onlyati_datastore::enums::{KeyType, ValueType, ListType};

let mut db = onlyati_datastore::controller::Database::new("root".to_string()).unwrap();

let list: Vec<(KeyType, ValueType)> = vec![
    (KeyType::Record("/root/status/sub1".to_string()), ValueType::RecordPointer("OK".to_string())),
    (KeyType::Record("/root/status/sub2".to_string()), ValueType::RecordPointer("NOK".to_string())),
    (KeyType::Record("/root/network/dns".to_string()), ValueType::RecordPointer("OK".to_string())),
    (KeyType::Record("/root/network/www".to_string()), ValueType::RecordPointer("NOK".to_string())),
];

for (key, value) in list {
    db.insert(key, value).expect("Failed to insert");
}

let full_list = db.list_keys(KeyType::Record("/root".to_string()), ListType::All).expect("Failed to get all keys");
assert_eq!(true, full_list.len() == 4);
```