use std::fmt;

use crate::resp::RESP;

#[derive(Debug, PartialEq)]
pub enum ServerError {
    CommandInternalError(String),
    CommandSyntaxError(String),
    CommandNotAvailable(String),
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
            ServerError::CommandInternalError(string) => {
                write!(f, "Internal Error while processing {}", string)
            }
            ServerError::CommandSyntaxError(string) => {
                write!(f, "Syntax Error while processing {}", string)
            }
            ServerError::IncorrectData => write!(f, "Data received from stream is incorrect!"),
            ServerError::StorageNotInitialized => write!(f, "Storage has not been initialized!"),
            ServerError::CommandNotAvailable(string) => {
                write!(f, "command not available {}", string)
            }
        }
    }
}

pub type ServerResult = Result<ServerValue, ServerError>;
