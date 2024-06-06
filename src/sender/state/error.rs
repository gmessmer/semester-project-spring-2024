use prusti_contracts::*;
use crate::types::MyResult;

// use super::Error::{self, *};
use super::SenderError::{self, *};

pub type Result<T> = MyResult<T, SenderError>;
