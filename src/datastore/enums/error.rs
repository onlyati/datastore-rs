///
/// Possible error types that database can return
///
#[derive(Debug)]
pub enum ErrorKind {
    /// The root name in the key does not match with the root table name
    InvalidRoot(String),

    /// Wrong key is specified, reason in the message
    InvalidKey(String),

    /// Oops, it should not happen
    InternalError(String),

    /// HookManager is not activated
    InactiveHookManager,

    /// Send log errors back
    LogError(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let response = match self {
            Self::InvalidKey(message) => format!("Invalid key: {message}"),
            Self::InvalidRoot(message) => format!("Invalid root: {message}"),
            Self::InternalError(message) => format!("Internal error: {message}"),
            Self::InactiveHookManager => format!("Inacvite hook manager: database is not subscried"),
            Self::LogError(message) => format!("LogError: {}", message),
        };
        return write!(f, "{}", response);
    }
}
