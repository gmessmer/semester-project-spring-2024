use prusti_contracts::*;
// use super::Error::{self, *};
use super::SenderError::{self, *};

#[derive(Clone, Debug)]
pub enum Result<T> {
    Value(T),
    Error(SenderError),
}



impl<T> Result<T> {
    #[pure]
    pub fn is_ok(&self) -> bool {
        match self {
            Result::Value(_) => true,
            Result::Error(_) => false,
        }
    }

    #[pure]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    #[pure]
    #[requires(self.is_ok())]
    pub fn unwrap(self) -> T {
        match self {
            Result::Value(value) => value,
            Result::Error(_) => panic!(),
        }
    }

    #[requires(self.is_err())]
    pub fn unwrap_err(self) -> SenderError {
        match self {
            Result::Value(_) => panic!(),
            Result::Error(e) => e.clone(),
        }
    }

    // #[ensures(result.is_ok() == res.is_ok())]
    // pub fn from(res: io::Result<T>) -> Self {
    //     match res {
    //         Ok(value) => Result::Value(value),
    //         Error(err) => Result::Error(err.to_string()),
    //     }
    // }
}