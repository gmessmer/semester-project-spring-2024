use prusti_contracts::*;

use crate::types::MyResult;

#[derive(Clone)]
pub enum ReceiverError {
    SocketError,
    RecvError,
}

impl ReceiverError {
    pub fn to_string(&self) -> String {
        match self {
            ReceiverError::SocketError => String::from("SocketError"),
            ReceiverError::RecvError => String::from("RecvError"),
        }
    }
}

pub type Result<T> = MyResult<T, ReceiverError>;