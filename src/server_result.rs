use std::fmt;

use crate::resp::RESP;

#[derive(Debug, PartialEq)]
pub enum ServerError {
    CommandError,
    IncorrectData,
    StorageNotInitialized,
}

#[derive(Debug, PartialEq)]
pub enum ServerValue {
    RESP(RESP),
}

#[derive(Debug, PartialEq)]
pub enum ServerMessage {
    Data(ServerValue),
    Error(ServerError),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CommandError => write!(f, "Error while processing!"),
            ServerError::IncorrectData => write!(f, "Data received from stream is incorrect!"),
            ServerError::StorageNotInitialized => write!(f, "Storage has not been initialized!"),
        }
    }
}

pub type ServerResult = Result<ServerValue, ServerError>;
