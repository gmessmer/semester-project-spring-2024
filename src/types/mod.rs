use std::fmt::{Debug, Display};

use super::*;

pub mod array;
pub mod list;
pub mod socket;
pub mod messaging;
// trait ToString {
//     fn to_string(&self) -> String;
// }

#[derive(Clone)]
pub enum MyResult<T, E> 
where E: Clone {
    Value(T),
    Error(E),
}

impl<T, E> MyResult<T, E> 
where E: Clone {
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

    #[requires(self.is_ok())]
    pub fn unwrap(self) -> T {
        match self {
            MyResult::Value(value) => value,
            MyResult::Error(e) => unreachable!(),
        }
    }

    #[pure]
    #[requires(self.is_ok())]
    pub fn unwrap_as_ref(&self) -> &T {
        match self {
            MyResult::Value(value) => value,
            MyResult::Error(e) => unreachable!(),
        }
    }

    #[requires(self.is_err())]
    pub fn unwrap_err(self) -> E {
        match self {
            MyResult::Value(_) => panic!(),
            MyResult::Error(e) => e.clone(),
        }
    }
}