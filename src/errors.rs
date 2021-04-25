use serde::{de, ser};
use std;
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, RstzError>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Clone, Debug, PartialEq)]
pub enum RstzError {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    // Zero or more variants that can be created directly by the Serializer and
    // Deserializer without going through `ser::Error` and `de::Error`. These
    // are specific to the format, in this case JSON.
    Eof,
    StdIoError(String),
    NoneError,
}

impl ser::Error for RstzError {
    fn custom<T: Display>(msg: T) -> Self {
        RstzError::Message(msg.to_string())
    }
}

impl de::Error for RstzError {
    fn custom<T: Display>(msg: T) -> Self {
        RstzError::Message(msg.to_string())
    }
}

impl Display for RstzError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RstzError::Message(msg) => formatter.write_str(msg),
            RstzError::Eof => formatter.write_str("unexpected end of input"),
            RstzError::StdIoError(msg) => formatter.write_str(msg),
            RstzError::NoneError => formatter.write_str("Unespected option None value"),
        }
    }
}

impl std::error::Error for RstzError {}

impl From<std::io::Error> for RstzError {
    fn from(e: std::io::Error) -> Self {
        RstzError::StdIoError(e.to_string())
    }
}

impl RstzError {
    pub fn new(msg: &str) -> RstzError {
        RstzError::Message(msg.to_string())
    }

    pub fn from_none() -> RstzError {
        RstzError::NoneError
    }
}
