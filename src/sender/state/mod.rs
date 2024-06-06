// use super::*;
pub mod error;


use std::time::Duration;
use std::u8::MAX;

use prusti_contracts::*;
use rand::{random, seq};
use crate::types::socket::{Socket, SocketError};
use crate::DATA_SIZE;
use crate::types::{MyResult, messaging::Packet};
use self::error::Result;

#[derive(Clone)]
pub enum SenderError {
    SendError{data: u8},
    SocketError,
    NoResponse,
    IllegalState,
    Timeout,
    BadTimeoutInput,
}

pub struct Error {
  kind: ErrorKind,
  pkt: Option<Packet>
}

pub enum ErrorKind {
  SendError,
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
      Result::Value(Ready {seq: 0, socket})
    },
    MyResult::Error(_) => Result::Error(SocketError)
  }
}


impl Ready {
  // #[ensures(result.is_ok() ==> snap(&self).socket.nsent() + 1 == result.unwrap().socket.nsent())]
  pub fn send(mut self, data: u8) -> Result<Pending> {
    let pkt = Packet {seq: self.seq, data};
    let res = self.socket.send_pkt(&pkt);
    match res {
      MyResult::Value(()) => Result::Value(Pending {seq: self.seq, socket: self.socket, data}),
      MyResult::Error(_) => Result::Error(SendError{data})
    }
  }
}

impl Pending {

  // #[ensures(result.is_ok() ==> result.unwrap().socket.nrecv() == snap(&self).socket.nrecv() + 1)]
  //$$ Delivered
  // #[ensures(result[0].is_ok() && result[1] ==> result[0].unwrap().socket.nrecv() == snap(&self).socket.nrecv() + 1)]
  #[ensures(result.0.is_ok() && result.1 ==> snap(&self).seq + 1 == result.0.unwrap_as_ref().seq)]
  //$$ Timeout
  // #[ensures(result[0].is_ok() && !result[1] ==> result[0].unwrap().socket.nrecv() == snap(&self).socket.nrecv())]
  #[ensures(result.0.is_ok() && !result.1 ==> snap(&self).seq == result.0.unwrap_as_ref().seq)]
  //$$ Error
  // Might add something to socket state to check if it is in illegal state
  // #[ensures(!result.0.is_ok() ==> result.0.unwrap().socket.nrecv() == snap(&self).socket.nrecv())]

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
    if timeout.as_millis() <= 0 {
      // In that case, didn't wait at all
      return (Result::Value(Ready {socket: self.socket, seq: self.seq}), false);
    }

    let r = self.socket.set_read_timeout(timeout);
    if r.is_err() {
      return (Result::Error(BadTimeoutInput), false);
    }

    let next_seq = self.seq + 1;
    let seq = self.seq;
    let t0 = std::time::Instant::now();
    let res = self.socket.recv_pkt();

    match res {
      MyResult::Value(pkt) => {
        if pkt.seq == seq {
          (Result::Value(Ready {socket: self.socket, seq: next_seq}), true)
        } else {
          let delta = std::time::Instant::now().duration_since(t0);
          let timeout1 = timeout - delta;
          self.wait_deliver(timeout1)
        }
      },
      MyResult::Error(SocketError::Timeout) => {
        (Result::Value(Ready {socket: self.socket, seq}), false)
      },
      MyResult::Error(_) => (Result::Error(SenderError::NoResponse), false)
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

use crate::types::socket::ServerSocket;

use super::*;

  #[test]
  fn test_send() {
    let remote_addr = "localhost:8080".to_string();
    print!("Connecting to {}\n", remote_addr);
    let tj = thread::spawn(|| {
      run_receiver(1, Duration::from_secs(3))
    });
    thread::sleep(Duration::from_secs(2));
    let res = connect(remote_addr).unwrap();
    let seq = res.seq;
    print!("Connected\n");
    let data = 1;
    let res = res.send(data);
    match res {
      Result::Value(pending) => {
        print!("Sent data {} with seq {}\n", data, seq);
        match pending.wait_deliver(Duration::from_secs(10)) {
          (Result::Value(ready), true) => print!("Data delivered, terminating sender...\n"),
          (Result::Value(ready), false) => print!("Timeout, resending required...\n"),
          _ => print!("Other error....\n"),
        }
      },
      Result::Error(e) => print!("Error when sending...\n")
    }
    tj.join().unwrap();
  }
    
  fn run_receiver(step: usize, timeout: Duration) {
    let mut r = ServerSocket::bind("localhost:8080".to_string())
      .unwrap()
      .accept()
      .unwrap();
    let res = r.set_read_timeout(timeout);
    assert!(res.is_ok());
    print!("Receiver ready\n");
    for _ in 0..step {
      let res = r.recv_pkt();
      match res {
        MyResult::Value(pkt) => {
          print!("Received: {}\n", pkt.to_string());
          let response_pkt = pkt.to_ack();
          r.send_pkt(&response_pkt).unwrap();
          print!("Sent ACK for data {}\n", pkt.data);
        },
        MyResult::Error(e) => print!("Error receving data: {}\n", e.to_string())
      }
    }
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
  }

