use core::fmt;

use crate::{request::Request, server_result::ServerError};

#[derive(Debug)]
pub enum ConnectionMessage {
    Request(Request),
}

#[derive(Debug)]
pub enum ConnectionError {
    ServerError(ServerError),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::ServerError(e) => {
                write!(f, "{}", format!("Server error:{}", e))
            }
        }
    }
}
