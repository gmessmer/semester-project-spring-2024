// use super::*;
pub mod error;


use std::time::Duration;
use std::u8::MAX;

use prusti_contracts::*;
use rand::{random, seq};
use crate::types::socket::{Socket, SocketError};
use crate::DATA_SIZE;
use crate::types::MyResult;
use self::error::Result::{self, *};

#[derive(Clone, Debug)]
pub enum SenderError {
    SendError{data: u8},
    SocketError,
    NoResponse,
    IllegalState,
    Timeout,
    BadTimeoutInput,
}

use SenderError::*;

pub trait ReadyTrait {
  fn send(self, data: u8) -> Result<Pending>;
}

pub struct Connect {
  seq: u8,
  remote_addr: String
}

pub struct Ready {
  seq: u8,
  socket: Socket
}

pub struct Pending {
  seq: u8,
  socket: Socket,
  data: u8
}


pub fn connect(remote_addr: String) -> Result<Ready> {
  let socket = Socket::connect(remote_addr);
  match socket {
    MyResult::Value(socket) => {
      let seq = random::<u8>();
      Value(Ready {seq, socket})
    },
    MyResult::Error(_) => Error(SocketError)
  }
}


impl Ready {
  #[ensures(result.is_ok() ==> snap(&self).socket.nsent() + 1 == result.unwrap().socket.nsent())]
  pub fn send(mut self, data: u8) -> Result<Pending> {
    let res = self.socket.send(data);
    match res {
      MyResult::Value(bs) => {
        if bs == DATA_SIZE {
          Value(Pending {seq: self.seq, socket: self.socket, data})
        } else {
          Error(SendError{data})
        }
      },
      MyResult::Error(_) => Error(SendError{data})
    }
  }
}

impl Pending {

  // #[ensures(result.is_ok() ==> result.unwrap().socket.nrecv() == snap(&self).socket.nrecv() + 1)]
  //$$ Delivered
  // #[ensures(result[0].is_ok() && result[1] ==> result[0].unwrap().socket.nrecv() == snap(&self).socket.nrecv() + 1)]
  #[ensures(result[0].is_ok() && result[1] ==> snap(&self).seq == (result[0].seq + 1) % std::u8::MAX)]
  //$$ Timeout
  // #[ensures(result[0].is_ok() && !result[1] ==> result[0].unwrap().socket.nrecv() == snap(&self).socket.nrecv())]
  #[ensures(result[0].is_ok() && !result[1] ==> snap(&self).seq == result[0].seq)]
  //$$ Error
  // Might add something to socket state to check if it is in illegal state
  #[ensures(!result[0].is_ok() ==> result[0].unwrap().socket.nrecv() == snap(&self).socket.nrecv())]

  // Define named predicate for result[0].unwrap().socket.nrecv() == snap(&self).socket.nrecv() + 1) and 
  // result[0].unwrap().socket.nrecv() == snap(&self).socket.nrecv()) so that it can be parsed
  // the first would transition from "sent" to "delivered" on network, the other would not do anything 
  // Also might require defining names for possible outcomes (eg. "Delivered, Timeout")

  pub fn wait_deliver(mut self, timeout: Duration) -> (Result<Ready>, bool) {
    // Handling timeout done with bool at the moment
    // Could be replaced by Result<Ready, Error> in the future
    // Or could introduce a field from_timeout to Ready
    // Or could introduce a struct Timeout that implements a trait same as Ready MIGHT BE THE BEST OPTION
    // BUT hard for Prusti specification,
    // TBD 
    let r = self.socket.set_read_timeout(timeout);
    if r.is_err() {
      return (Error(BadTimeoutInput), false);
    }
    let seq = self.seq.clone();
    let t0 = std::time::Instant::now();
    let res = self.socket.recv();
    match res {
      MyResult::Value(n) => {
        if n == seq {
          let next_seq = (seq + 1) % std::u8::MAX;
          (Value(Ready {socket: self.socket, seq: next_seq}), true)
        } else {
          let delta = std::time::Instant::now().duration_since(t0);
          let timeout1 = timeout - delta;
          if timeout1.as_millis() > 0 {
            self.wait_deliver(timeout1)
          } else {
            (Value(Ready {socket: self.socket, seq}), false)
          }
        }
      },
      MyResult::Error(SocketError::Timeout) => {
        (Value(Ready {socket: self.socket, seq}), false)
      },
      MyResult::Error(_) => (Error(SenderError::NoResponse), false)
    }
  }
}

impl SenderError {
  fn to_string(&self) -> String {
    match self {
      SendError{data} => "Failed to send data: ".to_string() + &data.to_string(),
      SocketError => "Failed to connect".to_string(),
      NoResponse => "No response".to_string(),
      IllegalState => "Illegal state".to_string(),
      Timeout => "Timeout".to_string(),
      BadTimeoutInput => "BadTimeoutInput".to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{io::{Read, Write}, net::TcpListener, thread, time::Duration};

use super::*;

  #[test]
  fn test_send() {
    let remote_addr = "localhost:8080".to_string();
    print!("Connecting to {}\n", remote_addr);
    let tj = thread::spawn(|| {
      let listener = TcpListener::bind("localhost:8080").unwrap();
      let (mut stream, addr) = listener.accept().unwrap();
      print!("Accepted connection from {}\n", addr);
      let mut buffer = [0; 1];
      let n = stream.read(&mut buffer);
      if n.is_err() {
        panic!("Error reading...\n")
      }
      print!("Received {}\n", buffer[0]);
      stream.write(&buffer);
      print!("Sent ACK\n");
    });
    thread::sleep(Duration::from_secs(2));
    let res = connect(remote_addr);
    print!("Connected\n");
    let data = 1;
    let res = res.unwrap().send(data);
    match res {
      Value(pending) => {
        print!("Sent data {}\n", data);
        match pending.wait_deliver(Duration::from_secs(10)) {
          (Value(ready), true) => print!("Data delivered, terminating sender...\n"),
          (Value(ready), false) => print!("Timeout, resending required...\n"),
          _ => print!("Other error....\n"),
        }
      },
      Error(e) => print!("Error when sending...\n")
    }
    
    
    // match res {
    //   Value(ready) => {
    //     let res = ready.send(1);
    //     match res {
    //       Value(pending) => {
    //         let res = pending.wait_deliver();
    //         match res {
    //           Value(_) => {},
    //           Error(e) => panic!("Error: {}", e.to_string())
    //         }
    //       },
    //       Error(e) => panic!("Error: {}", e.to_string())
    //     }
    //   },
    //   Error(e) => panic!("Error: {}", e.to_string())
    // }
    tj.join().unwrap();
  }
}

