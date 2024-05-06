use std::fmt::{Debug, Display};

use super::*;

pub mod array;
pub mod socket;

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

    #[pure]
    #[requires(self.is_ok())]
    pub fn unwrap(self) -> T {
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

pub struct Packet {
    seq: u8,
    data: u8,
}

pub const BYTES_PER_PACKET: usize = 2;

impl Packet {
    pub fn new(seq: u8, data: u8) -> Self {
        Packet { seq, data }
    }

    pub fn marshall(&self) -> [u8; 2] {
        [self.seq, self.data]
    }

    pub fn unmarshall(data: [u8; 2]) -> Self {
        Packet { seq: data[0], data: data[1] }
    }

    pub fn seq(&self) -> u8 {
        self.seq
    }

    pub fn data(&self) -> u8 {
        self.data
    }
}