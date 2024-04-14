use super::*;

mod array;
mod socket;

pub enum MyResult<T> {
    Value(T),
    Error(String),
}

impl<T> MyResult<T> {
    #[pure]
    pub fn is_ok(&self) -> bool {
        match self {
            MyResult::Value(_) => true,
            MyResult::Error(_) => false,
        }
    }

    #[pure]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    #[pure]
    #[requires(self.is_ok())]
    pub fn unwrap(&self) -> &T {
        match self {
            MyResult::Value(value) => value,
            MyResult::Error(msg) => panic!("{}", msg),
        }
    }

    #[requires(self.is_err())]
    pub fn unwrap_err(&self) -> &str {
        match self {
            MyResult::Value(_) => panic!(),
            MyResult::Error(msg) => msg,
        }
    }

    #[ensures(result.is_ok() == res.is_ok())]
    pub fn from(res: io::Result<T>) -> Self {
        match res {
            Ok(value) => MyResult::Value(value),
            Err(err) => MyResult::Error(err.to_string()),
        }
    }
}