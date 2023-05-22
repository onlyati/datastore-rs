# OnlyAti.Datastore :construction: Under development :construction:
 
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
| network | ------+    +------+
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

## Sample code to use the built-in thread server

There is a provided function that creates a thread, initialize database then return with a `std::sync::mpsc::Sender` so other thread can send request.
This a simple method to initialize this database, communication can be done by using channels. Hook manager can also be initailized on this way of option is provided.

```rust
use onlyati_datastore::datastore::{
    enums::{error::ErrorKind, DatabaseAction, pair::ValueType},
    utilities::{start_datastore, self},
};
use onlyati_datastore::hook::utilities::start_hook_manager;
use onlyati_datastore::logger::utilities::start_logger;
use std::sync::{Arc, Mutex};

let (hook_sender, _) = start_hook_manager();
let (logger_sender, _) = start_logger(&"/tmp/tmp-datastore-log.txt".to_string());

let hook_sender = Arc::new(Mutex::new(hook_sender));
let logger_sender = Arc::new(Mutex::new(logger_sender));

// Start a new database with active hook manager
let (sender, _) = start_datastore("root".to_string(), Some(hook_sender.clone()), Some(logger_sender));

// Send a POST request to specified address when records updated within /root/status
let (tx, rx) = utilities::get_channel_for_hook_set();
let action = DatabaseAction::HookSet(tx, "/root/network".to_string(), "http://127.0.0.1:3031".to_string());
sender.send(action).expect("Failed to send hook request");

rx.recv().expect("Failed to received response").expect("Bad request");

// Add a new pair
let (tx, rx) = utilities::get_channel_for_set();
let set_action = DatabaseAction::Set(tx, "/root/network/server1".to_string(), "ok".to_string());

sender.send(set_action).expect("Failed to send the request");
rx.recv().unwrap().unwrap();

// Get the pair
let (tx, rx) = utilities::get_channel_for_get();
let get_action = DatabaseAction::Get(tx, "/root/network/server1".to_string());

sender.send(get_action).expect("Failed to send the get request");
let data = rx.recv().expect("Failed to receive message").expect("Failed to get data");
assert_eq!(ValueType::RecordPointer("ok".to_string()), data);
```

## Sample code to run without built-in thread

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

For more samples check `src/tests` direcotry.
