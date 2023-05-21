#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::logger::{
        enums::{LogItem, LoggerAction, LoggerResponse},
        utilities::{start_logger, get_channel_for_log_write},
        LoggerManager,
    };

    #[test]
    fn test_log() {
        let path = Path::new("/tmp/datastore-log.txt");
        if path.exists() {
            std::fs::remove_file(path).expect("Failed to delete temp log");
        }

        let mut manager = LoggerManager::new(path);

        let result = manager.start();
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::GetHook("/root/status/server1", "alive"));
        assert_eq!(true, result.is_ok());

        let result = manager.stop();
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::GetHook("/root/status/server9", "alive"));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_log2() {
        let path = Path::new("/tmp/datastore-log2.txt");
        if path.exists() {
            std::fs::remove_file(path).expect("Failed to delete temp log");
        }

        // Start logger and write a line
        let mut manager = LoggerManager::new(path);

        let result = manager.start();
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::GetHook("/root/tickets/345", "open"));
        assert_eq!(true, result.is_ok());

        let result = manager.write(LogItem::GetHook("/root/tickets/346", "open"));
        assert_eq!(true, result.is_ok());

        // Suspend the logger: file is closed and every message will be buffered
        let result = manager.suspend();
        assert_eq!(true, result.is_ok());

        let content = std::fs::read_to_string(path).expect("Failed to open file for line counting");
        let count1: Vec<&str> = content.lines().collect();
        let count1 = count1.len();

        // Still should be ok, but instead of file, it is buffered in memory
        let result = manager.write(LogItem::GetHook("/root/tickets/345", "close"));
        assert_eq!(true, result.is_ok());

        // Let's check the number of lines in file, it should be the same
        let content = std::fs::read_to_string(path).expect("Failed to open file for line counting");
        let count2: Vec<&str> = content.lines().collect();
        let count2 = count2.len();

        assert_eq!(count1, count2);

        // Now make a resume then close the file (so it can be read)
        let result = manager.resume();
        assert_eq!(true, result.is_ok());

        let result = manager.stop();
        assert_eq!(true, result.is_ok());

        // Check line numbers again, should be more with one
        let content = std::fs::read_to_string(path).expect("Failed to open file for line counting");
        let count3: Vec<&str> = content.lines().collect();
        let count3 = count3.len();

        assert_eq!(count2 + 1, count3);
    }

    #[test]
    fn test_log3() {
        let path = Path::new("/tmp/datastore-log2.txt");
        if path.exists() {
            std::fs::remove_file(path).expect("Failed to delete temp log");
        }

        let (sender, _) = start_logger(&"/tmp/datastore-log2.txt".to_string());

        let action = LoggerAction::WriteAsync(vec![LogItem::SetKey("/root/test1", "something")]);
        sender.send(action).expect("Failed to send the request");

        let (tx, rx) = get_channel_for_log_write();

        let action = LoggerAction::Write(tx, vec![
            LogItem::GetKey("/root/test/1"),
            LogItem::GetKey("/root/test/2"),
            LogItem::GetKey("/root/test/3"),
        ]);
        sender.send(action).expect("Failed to send the request");

        let response = rx.recv().expect("Failed to receive reply");
        assert_eq!(LoggerResponse::Ok, response);

        let content = std::fs::read_to_string(path).expect("Failed to open file for line counting");
        let count: Vec<&str> = content.lines().collect();
        let count = count.len();

        assert_eq!(4, count);
    }
}
