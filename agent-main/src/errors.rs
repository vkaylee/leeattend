use std::fmt;

#[derive(Debug)]
pub enum AgentError {
    ConnectionTimeout(String),
    ConnectionFailed(String),
    DeviceNotConnected,
    ProtocolError(String),
    DatabaseError(String),
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionTimeout(msg) => write!(f, "Connection timeout: {msg}"),
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            Self::DeviceNotConnected => write!(f, "Device not connected"),
            Self::ProtocolError(msg) => write!(f, "Protocol error: {msg}"),
            Self::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Other(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl std::error::Error for AgentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AgentError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<String> for AgentError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for AgentError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}
