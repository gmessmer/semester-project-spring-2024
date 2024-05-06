use std::fmt::Display;

use super::*;

#[derive(Clone)]
pub struct Array<T> {
    data: Vec<T>,
    len: usize,
}
type MyResult<T> = crate::types::MyResult<T, String>;
// TODO: change to linked list for more Prusti support


impl<T> Array<T> {

    #[pure]
    pub fn len(&self) -> usize {
        self.len
    }

    #[ensures(result.len() == 0)]
    pub fn new() -> Self {
        Array { data: Vec::new(), len: 0 }
    }

    // #[requires(self.len() < usize::MAX)]
    #[ensures(result.is_ok() ==> self.len() == old(self.len()) + 1)]
    // #[ensures(result.is_ok() ==> self.last().unwrap() == &value)]
    #[ensures(!result.is_ok() ==> self.len() == old(self.len()))]

    pub fn push(&mut self, value: T) -> MyResult<()> {
        if self.len < usize::MAX {
            self.data.push(value);
            self.len += 1;
            return MyResult::Value(());
        }
        MyResult::Error("Array is full".to_string())
    }

    #[pure]
    pub fn can_push(&self) -> bool {
        self.len < usize::MAX
    }

    #[trusted]
    pub fn pop(&mut self) -> Option<T> {
        match self.data.pop() {
            Some(value) => {
                self.len -= 1;
                Some(value)
            },
            None => None,
        }
    }

    #[pure]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }



}