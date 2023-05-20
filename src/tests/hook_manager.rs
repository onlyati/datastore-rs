#[cfg(test)]
mod tests {
    use std::io::prelude::*;

    use crate::hook::{utilities, HookManager};

    #[test]
    fn test_hook_manager() {
        let mut manager = HookManager::new();

        let result = manager.add(
            "/root/status".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.add(
            "/root/status".to_string(),
            "http://127.0.0.1:3032".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.add(
            "/root/status".to_string(),
            "http://127.0.0.1:3032".to_string(),
        );
        assert_eq!(true, result.is_err());

        let result = manager.add(
            "/root/status".to_string(),
            "http://127.0.0.1:3033".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.add(
            "/root/arpa".to_string(),
            "http://127.0.0.1:3031".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.remove(
            "/root/status".to_string(),
            "http://127.0.0.1:3033".to_string(),
        );
        assert_eq!(true, result.is_ok());

        let result = manager.list(&"/root".to_string());
        assert_eq!(2, result.len());

        let result = manager.list(&"/root/stat".to_string());
        assert_eq!(1, result.len());

        let result = manager.list(&"/root/no_exist".to_string());
        assert_eq!(0, result.len());

        // Start a dummy TCP listenere for testing
        std::thread::spawn(|| {
            let listener = std::net::TcpListener::bind("127.0.0.1:3031")
                .expect("Failed to listen on 127.0.0.1:3031");
            println!("Start to listen");
            while let Ok(stream) = listener.accept() {
                let mut stream = stream.0;
                stream.set_read_timeout(None).unwrap();
                let buf_reader = std::io::BufReader::new(&stream);
                let mut http_request = String::new();
                for byte in buf_reader.bytes() {
                    match byte {
                        Ok(byte) => {
                            let char = byte as char;
                            http_request.push(char);
                            stream
                                .set_read_timeout(Some(std::time::Duration::new(0, 250)))
                                .unwrap();
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            println!("Unexpected error: {:?}", e);
                            let _ = stream.write_all(
                                b">Error\nInternal server error during stream reading\n",
                            );
                            panic!("TCP error");
                        }
                    }
                }

                println!("Request: {:#?}", http_request);
                stream
                    .write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())
                    .unwrap();
            }
            panic!("TCP listener has stopped");
        });

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let counter = manager
                .execute_hooks(&"/root/status/dns1".to_string(), &"okay".to_string())
                .await;
            assert_eq!(Some(2), counter);

            let counter = manager
                .execute_hooks(&"/root/no_exist".to_string(), &"okay".to_string())
                .await;
            assert_eq!(None, counter);

            let counter = manager
                .execute_hooks(
                    &"/root/arpa/server1".to_string(),
                    &"This is the value".to_string(),
                )
                .await;
            assert_eq!(Some(1), counter);

            // Wait some time until request are received
            tokio::time::sleep(tokio::time::Duration::new(2, 0)).await;
        });
    }

    #[test]
    fn hook_manager_with_datastore() {
        let (sender, _) = utilities::start_hook_manager();
        let (sender) =
            crate::datastore::utilities::start_datastore("root".to_string(), Some(sender));
    }
}
