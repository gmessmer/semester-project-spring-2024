use types::list::List;
use types::messaging::ack_with_seq;

use prusti_contracts::predicate;
use super::*;
mod error;
use crate::types::socket::*;
use self::types::messaging::Packet;
use self::{types::MyResult};
use self::error::ReceiverError::*;
use self::error::Result;

pub struct Ready {
  pub socket: ServerSocket,
}

pub struct Listening {
  seq: u8,
  socket: Socket,
  delivered: List<u8>
  // buffer: Array<u8>
}

pub struct Deliver {
  socket: Socket,
  pkt: Packet,
  seq: u8,
  delivered: List<u8>
}


pub fn bind(src_addr: String) -> Result<Ready> {
  let socket = ServerSocket::bind(src_addr);
  match socket {
    MyResult::Value(socket) => Result::Value(Ready {socket}),
    MyResult::Error(_) => Result::Error(SocketError)
  }
}

impl Ready {
  pub fn accept(&self) -> Result<Listening> {
    let s = self.socket.accept();
    match s {
      MyResult::Value(socket) => Result::Value(Listening {socket, seq: 0, delivered: List::new()}),
      MyResult::Error(_) => Result::Error(SocketError)
    }
  }
}


impl Listening {

  fn valid_pkt(&self, pkt: &Packet) -> bool {
    pkt.seq == self.seq || pkt.seq == self.seq - 1
  }

  // fn valid_seq(&self, seq: u8) -> bool {
  //   seq == self.seq || seq == self.seq - 1
  // }

  /// Receive a packet and move to the next state if the packet is valid.
  /// Otherwise stays in Listening state.

  // $[NetworkIn(result.pkt.seq)]
  // $[Recursive]
  #[ensures(result.is_ok() ==> result.unwrap_as_ref().pkt.seq == self.seq || result.unwrap_as_ref().pkt.seq == self.seq - 1)]
  pub fn recv(mut self) -> Result<Deliver> {
      let res = self.socket.recv_pkt();
      match res {
        MyResult::Value(pkt) => {
          if pkt.seq == self.seq || pkt.seq == self.seq - 1 {
            MyResult::Value(Deliver { socket: self.socket, seq: self.seq, pkt, delivered: self.delivered})
          } else {
            self.recv()
          }
        },
        MyResult::Error(_) => return Result::Error(RecvError)
    }
  }


}

impl Deliver {

  /// Acknowledge the packet and move to the next state (Listening in case of success).
  #[ensures(result.is_ok() ==> result.unwrap_as_ref().seq == snap(&self).pkt.seq + 1)]
  #[ensures(result.is_ok() ==> result.unwrap_as_ref().delivered.is_union(&snap(&self).delivered, snap(&self).pkt.seq))]
  pub fn deliver(mut self) -> Result<Listening> {
    let next_seq = self.pkt.seq + 1;
    let ack = ack_with_seq(self.pkt.seq + 1);
    let res = self.socket.send_pkt(&ack);
    match res {
      MyResult::Value(_) => {
        Result::Value(Listening {socket: self.socket, seq: next_seq, delivered: self.delivered.add(self.pkt.seq)})
      },
      MyResult::Error(_) => Result::Error(SocketError)
    }
  }
}

#[cfg(test)]
mod tests {

    use std::{thread, time::Duration};

    use rand::random;

    use crate::{sender::{self, Sender}, types::{messaging::Packet, MyResult}};

    use super::{bind, error::Result, Listening, Socket};

  #[test]
  fn test_receiver_protocol() {
    let src_addr = "localhost:8080".to_string();
    let receiver = bind(src_addr.clone()).unwrap();
    let n = 3;

    let sj = thread::spawn(move || {
      run_client(src_addr, n)
    });
    
    print!("RECEIVER: Waiting for connections..\n");
    let mut l = receiver.accept().unwrap();
    print!("RECEIVER: Connection established\n");

    for _ in 0..n {
      l = run_server_once(l).unwrap();
    }
    sj.join().unwrap();
  }

  fn run_client(addr: String, n_data: usize) {
    let mut seq = 0;
    let mut s = Socket::connect(addr)
      .unwrap();
    print!("SENDER: Connected\n");
    thread::sleep(Duration::from_secs(3));
    for _ in 0..n_data {
      let data = rand::random::<u8>();
      let pkt = Packet::new(seq, data);
      s.send_pkt(&pkt).unwrap();
      print!("SENDER: Sent pkt with seq {}\n", pkt.seq);
      let resp = s.recv_pkt();
      match resp {
        MyResult::Value(pkt) => {
          print!("SENDER: Received ACK with seq {}\n", pkt.seq);
          assert!(pkt.seq == seq);
        },
        MyResult::Error(e) => print!("SENDER: Error when receiving ACK: {}\n", e.to_string())
      }
      seq += 1;
    } 
  }

  fn run_server_once(listening: Listening) -> Result<Listening> {
    let r = listening.recv();
      match r {
        Result::Value(deliver) => {
          let seq = deliver.pkt.seq;
          let data = deliver.pkt.data;
          print!("RECEIVER: Received data {} with seq {}\n", data, seq);
          match deliver.deliver() {
            Result::Value(listening) => {
              print!("RECEIVER: Delivered for seq {}\n", seq);
              Result::Value(listening)
            },
            Result::Error(e) => {
              print!("RECEIVER: Error when delivering: {}\n", e.to_string());
              Result::Error(e)
            },
          }
        },
        Result::Error(e) => {
          print!("RECEIVER: Error when receiving: {}\n", e.to_string());
          Result::Error(e)
        }
      }
  }

}