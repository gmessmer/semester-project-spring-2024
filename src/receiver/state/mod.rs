use super::*;
mod error;
use crate::types::array::Array;
use crate::types::socket::*;
use self::{error::*, types::MyResult};
use self::error::ReceiverError::*;
use self::error::Result::{self, *};
pub struct Ready {
  socket: ServerSocket 
}

pub struct Listening {
  socket: Socket,
  // buffer: Array<u8>
}

pub struct Deliver {
  socket: Socket,
  data: u8
}


pub fn bind(src_addr: String) -> Result<Ready> {
  let socket = ServerSocket::bind(src_addr);
  match socket {
    MyResult::Value(socket) => Value(Ready {socket}),
    MyResult::Error(_) => Error(SocketError)
  }
}

impl Ready {
  fn accept(&self) -> Result<Listening> {
    let s = self.socket.accept();
    match s {
      MyResult::Value(socket) => Value(Listening {socket}),
      MyResult::Error(_) => Error(SocketError)
    }
  }
}

impl Listening {
  fn recv(mut self) -> Result<Deliver> {
      let res = self.socket.recv();
      match res {
        MyResult::Value(data) => Value(Deliver {socket: self.socket, data}),
        MyResult::Error(_) => return Error(RecvError)
    }
  }
}

impl Deliver {
  fn deliver(mut self) -> Result<Listening> {
    let res = self.socket.send(self.data);
    match res {
      MyResult::Value(_) => Value(Listening {socket: self.socket}),
      MyResult::Error(_) => Error(SocketError)
    }
  }
}

#[cfg(test)]
mod tests {

    use std::{thread, time::Duration};

    use crate::types::MyResult;

    use super::{bind, error::Result, Socket};

  #[test]
  fn test_receiver_protocol() {
    let src_addr = "localhost:8080".to_string();
    let receiver = bind(src_addr.clone()).unwrap();
    let sj = thread::spawn(move || {
      run_client(src_addr)
    });
    let s = receiver.accept().unwrap();
    let r = s.recv();
    match r {
      Result::Value(deliver) => {
        let data = deliver.data.clone();
        match deliver.deliver() {
          Result::Value(_) => print!("Delivered for data {}\n", data),
          Result::Error(_) => print!("Error when delivering"),
        }
      },
      Result::Error(_) => print!("Error when receiving..\n")
    };
    sj.join();
  }

  fn run_client(addr: String) {
    let data = 10;
    let mut s = Socket::connect(addr)
      .unwrap();
    thread::sleep(Duration::from_secs(3));
    s.send(data);
    let r = s.recv();
    match r {
      MyResult::Value(v) => print!("Received ACK {} for data {}\n", v, data),
      MyResult::Error(_) => print!("SocketError\n")
    }
  }
}