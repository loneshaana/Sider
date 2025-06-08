use std::fmt;

#[derive(Debug, PartialEq)]
pub enum StorageError {
    IncorrectRequest,
    CommandSyntaxError(String),
    CommandInternalError(String),
    CommandNotAvailable(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::IncorrectRequest => {
                write!(f, "the client sent and incorrect request!")
            }
            StorageError::CommandNotAvailable(c) => {
                write!(f, "the requested command {} is not available", c)
            }
            StorageError::CommandSyntaxError(c) => {
                write!(f, "syntax error while processing {}", c)
            }
            StorageError::CommandInternalError(c) => {
                write!(f, "internal error while processing {}", c)
            }
        }
    }
}

pub type StorageResult<T> = Result<T, StorageError>;
