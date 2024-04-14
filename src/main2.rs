use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use prusti_contracts::*;

enum Result<T> {
    Ok(T),
    Error(String),
}

impl<T> Result<T> {
    #[pure]
    fn is_ok(&self) -> bool {
        match self {
            Result::Ok(_) => true,
            Result::Error(_) => false,
        }
    }

    #[pure]
    fn is_err(&self) -> bool {
        !self.is_ok()
    }

    #[ensures(self.is_ok())]
    #[ensures(matches!(result, Result::Value(_)))]
    fn unwrap(&self) -> &T {
        match self {
            Result::Ok(value) => value,
            Result::Error(msg) => panic!("{}", msg),
        }
    }

    #[ensures(self.is_err())]
    #[ensures(matches!(result, Result::Error(_)))]
    fn unwrap_err(&self) -> &str {
        match self {
            Result::Ok(_) => panic!("called `Result::unwrap_err()` on a `Value` value"),
            Result::Error(msg) => msg,
        }
    }
}

#[derive(Clone)]
pub struct Link {
    pub src: String,
    pub dst: String,
    pub capacity: usize,
}

struct Init {}