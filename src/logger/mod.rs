use chrono::{DateTime, Utc};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use self::enums::{LogItem, LogState};

pub mod enums;
pub mod utilities;

/// Logger manager main structure
/// 
/// There are 3 state fo logger:
/// - Open: File is open, so direct write is possible
/// - Close: File is closed, so direct write is not possible and every write request will be dismissed
/// - Suspended: File is closed, and every further write request will be buffered. Once it is resumed, those records will be written
///
/// # Examples
/// ```
/// use std::path::Path;
/// use onlyati_datastore::logger::{LoggerManager, enums::LogItem};
///
/// // Create a new logger instance that will run into a temp file
/// // Normally this point to a file that is not temporary
/// let mut logger = LoggerManager::new("/tmp/datastore-log-monday".to_string());
///
/// // Start the logger
/// logger.start().expect("Failed to start logger");
///
/// // Write some line
/// logger.write(LogItem::GetKey("/root/agent/procops/status".to_string())).expect("Failed to write");
/// logger.write(LogItem::SetKey("/root/agent/procops/status".to_string(), "unavailable".to_string())).expect("Failed to write");
///
/// // Stop the logger
/// logger.stop().expect("Failed to stop logger");
/// ```
/// 
/// For more details check `src/tests/logger.rs` file.
pub struct LoggerManager {
    path: String,
    state: LogState,
    file: Option<BufWriter<File>>,
    buffer: Vec<(DateTime<Utc>, LogItem)>,
}

impl LoggerManager {
    /// Allocate new logger
    pub fn new(path: String) -> Self {
        tracing::trace!("allocate new log manager with '{}' path", path);
        return LoggerManager {
            path,
            state: LogState::Close,
            file: None,
            buffer: Vec::new(),
        };
    }

    /// Open a buffer for the specified file
    /// After it, every write request will be directly written to file
    pub fn start(&mut self) -> Result<(), String> {
        match File::options()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(Path::new(&self.path))
        {
            Ok(file) => {
                tracing::trace!("log file is open");
                self.file = Some(BufWriter::new(file));
                self.state = LogState::Open;
                return Ok(());
            }
            Err(e) => {
                tracing::error!("failed to open log file: {}", e);
                return Err(format!("Failed to open log file: {}", e));
            }
        };
    }

    /// Close the buffer for the specified file
    /// After it, every write request is going to be failed and not buffered.
    pub fn stop(&mut self) -> Result<(), String> {
        match &mut self.file {
            Some(_) => {
                self.file = None;
                return Ok(());
            }
            None => return Err(String::from("Logger manager does not run")),
        }
    }

    /// Close the log file and do not write more into it.
    /// Instead buffer every message into memory.
    /// They will be written if the logging has resumed.
    pub fn suspend(&mut self) -> Result<(), String> {
        if self.file.is_some() {
            self.file = None;
        }
        self.state = LogState::Suspended;

        return Ok(());
    }

    /// Resume the logging means that those message which were buffered during suspended status will be written first.
    /// Then status will be LogState::Open again.
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != LogState::Suspended {
            return Err(String::from("Only possible resume from LogState::Suspend"));
        }

        if let Err(e) = self.start() {
            return Err(e);
        }

        for item in &self.buffer {
            if let Some(file) = &mut self.file {
                let line = format!("{} {}\n", item.0, item.1);
                if let Err(e) = file.write_all(line.as_bytes()) {
                    tracing::error!("failed to write log after a resume: {}", e);
                    return Err(format!("Failed to write log after a resume: {}", e));
                }
            }
        }

        self.buffer = Vec::new();

        self.state = LogState::Open;
        return Ok(());
    }

    /// Make a write reqest
    pub fn write(&mut self, item: LogItem) -> Result<(), String> {
        let now = Utc::now();

        match &mut self.state {
            // Logger is stopped
            LogState::Close => {
                return Err(String::from("Stream is closed, start required for logger"))
            }
            // Regular write to a file
            LogState::Open => {
                match &mut self.file {
                    Some(file) => {
                        let line = format!("{} {}\n", now, item);
                        match file.write_all(line.as_bytes()) {
                            Ok(_) => return Ok(()),
                            Err(e) => {
                                tracing::error!("error during log writing: {}", e);
                                return Err(format!("error during log writing: {}", e));
                            }
                        }
                    }
                    None => {
                        tracing::error!("wanted to write log while logging was not started");
                        return Err(String::from(
                            "wanted to write log while logging was not started",
                        ));
                    }
                }
            }
            // Buffer lines into memory
            LogState::Suspended => {
                self.buffer.push((now, item.clone()));
                return Ok(());
            }
        }
    }
}
