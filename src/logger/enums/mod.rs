use std::sync::mpsc::Sender;

/// Item for every action in datastore
#[derive(Clone, Debug)]
pub enum LogItem<'a> {
    SetKey(&'a str, &'a str),
    GetKey(&'a str),
    RemKey(&'a str),
    RemPath(&'a str),
    ListKeys(&'a str),
    SetHook(&'a str, &'a str),
    GetHook(&'a str, &'a str),
    RemHook(&'a str, &'a str),
    ListHooks(&'a str),
    HookExecute(&'a str, &'a Vec<String>)
}

impl<'a> std::fmt::Display for LogItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::SetKey(key, value) => format!("SetKey [ '{}', '{}' ]", key, value),
            Self::GetKey(key) => format!("GetKey [ '{}' ]", key),
            Self::RemKey(key) => format!("RemKey [ '{}' ]", key),
            Self::RemPath(key) => format!("RemPath [ '{}' ]", key),
            Self::ListKeys(key) => format!("ListKeys [ '{}' ]", key),
            Self::SetHook(prefix, link) => format!("SetHook [ '{}', '{}' ]", prefix, link),
            Self::GetHook(prefix, link) => format!("GetHook [ '{}', '{}' ]", prefix, link),
            Self::RemHook(prefix, link) => format!("RemHook [ '{}', '{}' ]", prefix, link),
            Self::ListHooks(prefix) => format!("ListHooks [ '{}' ]", prefix),
            Self::HookExecute(prefix, links) => format!("HookExecute [ '{}', '{:?}' ]", prefix, links),
        };
        return write!(f, "{}", text);
    }
}

/// Represent state of logger
#[derive(PartialEq)]
pub enum LogState {
    /// File is closed, no write is possible
    Close,

    /// File is open, can be written directly
    Open,

    /// File is closed, but writes are buffered in memory
    Suspended,
}

/// Types that can be sent back by using the `start_logger` utility
#[derive(PartialEq, Debug)]
pub enum LoggerResponse {
    /// Request is successfully done
    Ok,

    /// Something is wrong, see in message
    Err(String),
}

/// Enums for the `start_logger` utility taht can be used with an std::sync::mpsc::Sender<LoggerAction> sender.
pub enum LoggerAction<'a> {
    /// Close log file and buffer further message
    Suspend(Sender<LoggerResponse>),

    /// Open log file and write the buffered message
    Resume(Sender<LoggerResponse>),

    /// Write request
    Write(Sender<LoggerResponse>, Vec<LogItem<'a>>),
    WriteAsync(Vec<LogItem<'a>>),
}

impl<'a> std::fmt::Display for LoggerAction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Resume(_) => "Resume".to_string(),
            Self::Suspend(_) => "Suspend".to_string(),
            Self::Write(_, item) => format!("Write [ '{:?}' ]", item),
            Self::WriteAsync(item) => format!("Write [ '{:?}' ]", item),
        };
        return write!(f, "{}", text);
    }
}
