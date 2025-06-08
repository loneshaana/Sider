use core::fmt;
use std::{num::ParseIntError, string::FromUtf8Error};

#[derive(Debug, PartialEq)]
pub enum RESPError {
    FromUtf8,
    Unknown,
    WrongType,
    ParseInt,
    IncorrectLength(i32),
    OutOfBounds(usize),
}

pub type RESPResult<T> = Result<T, RESPError>;
pub type RESPLength = i32;

impl fmt::Display for RESPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RESPError::OutOfBounds(index) => write!(f, "Out of bounds at index {}", index),
            RESPError::FromUtf8 => write!(f, "Cannot convert from UTF-8"),
            RESPError::WrongType => write!(f, "Wrong prefix for RESP type"),
            RESPError::Unknown => write!(f, "Unknown format of RESP string"),
            RESPError::ParseInt => write!(f, "Cannot parse string to integer"),
            RESPError::IncorrectLength(_) => write!(f, "incorrect length of buk string"),
        }
    }
}

impl From<FromUtf8Error> for RESPError {
    fn from(_err: FromUtf8Error) -> Self {
        Self::FromUtf8
    }
}

impl From<ParseIntError> for RESPError {
    fn from(_err: ParseIntError) -> Self {
        Self::ParseInt
    }
}
