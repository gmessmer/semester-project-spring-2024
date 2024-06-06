use core::{panic, time};
use std::{process::{abort, exit}, task::ready, thread, time::Duration};

use self::{state::{Pending, Ready}, types::{array::Array, socket, MyResult::{self, *}}};

use super::*; 
mod state;
mod state0;



#[derive(Clone)]
enum SenderError {
  RemoteUnavailable,
  Timeout,
  SendFailed,
  IllegalState,
  OperationTimedOut,
}
use SenderError::*;
type Result<T> = crate::types::MyResult<T, SenderError>;

const SLEEP_TIME_SECONDS: u64 = 1;
const TIMEOUT_SECONDS: u64 = 120;
const ACK_TIMEOUT_SECONDS: u64 = 1;
const ACK_TIMEOUT: Duration = Duration::from_secs(ACK_TIMEOUT_SECONDS);
const INITIAL_WAIT_TIME_RECONNECT: u64 = 10;

pub struct Sender {
  remote_addr: String,
  // ready_state == None when ready_state is not ready to send
  ready_state: Option<Ready>,
  // pending_state: Option<Pending>,
  // sending: Option<u8>,
  // delivered: Array<u8>,
}

pub fn init(remote_addr: String) -> Sender {
  Sender {
    remote_addr,
    ready_state: None,
    // pending_state: None,
    // sending: None,
    // delivered: Array::new(),
  }
}

/// Setup the sender part of the perfect link with the remote address
/// This function always return an instance of Sender. Blocks until the remote is available.
#[ensures(result.is_connected())]
fn connect(remote_addr: String) -> Sender {
  let wait_time = Duration::from_secs(INITIAL_WAIT_TIME_RECONNECT);
  try_connect(remote_addr, wait_time)
}

/// Attempt to connect to the remote address. 
/// If the connection fails, retry after sleep_time.
/// If the connection is not established within timeout, return an error.
#[ensures(result.is_connected())]
fn try_connect(remote_addr: String, sleep_time: Duration) -> Sender {
  let res = state::connect(remote_addr.clone());
  match res {
    MyResult::Value(ready) => Sender {remote_addr, ready_state: Some(ready)},
    MyResult::Error(_) => {
      thread::sleep(sleep_time);
      try_connect(remote_addr, sleep_time * 2)
    }
  }
}

/// Recover from a sender error by reconnecting to the remote address
#[ensures(result.is_connected())]
fn recover(e: SenderError, dst: String) -> Sender {
  connect(dst)
}

impl Sender {
  #[pure]
  fn is_connected(&self) -> bool {
    self.ready_state.is_some()
  }

  /// Send data to the remote adress. If the data is not delivered within the timeout, resend the data with timeout doubled.
  /// If the connection is lost, reconnect and resend the data.
  #[requires(self.is_connected())]
  fn send_data(self, data: u8, timeout: Duration) -> Sender {
    assert!(self.ready_state.is_some());
    let remote_addr = self.remote_addr.clone();
    let res = self.ready_state.unwrap().send(data);
    match res {
      Value(pending) => {
        // Wait for the data to be delivered
        let res = pending.wait_deliver(timeout);
        match res {
          (Value(ready), true) => Sender {ready_state: Some(ready), remote_addr},
          (Value(ready), false) => {
            // Timeout => resend data
            let s = Sender {ready_state: Some(ready), remote_addr};
            s.send_data(data, timeout * 2) // double timeout
          },
          (Error(e), _) => recover(SendFailed, remote_addr).send_data(data, timeout)
        }
      },
      Error(_) => recover(SendFailed, remote_addr).send_data(data, timeout)
    }
  }

  #[ensures(result.is_connected())]
  fn connect_if_required(self) -> Sender {
    let remote_addr = self.remote_addr.clone();
    match self.is_connected() {
      true => self,
      false => connect(self.remote_addr)
    }
  }

  pub fn send(self, data: u8) -> Sender {
    // 1. Connect if not already connected
    let ack_timeout = Duration::from_secs(ACK_TIMEOUT_SECONDS);
    self.connect_if_required()
      // 2. Send data
      .send_data(data, ack_timeout)
  }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::types::{socket::{ServerSocket, Socket}, MyResult};

    use super::init;

    #[test]
    fn send_data() {
      let remote_addr = "localhost:8080".to_string();
      thread::spawn(|| {
        run_receiver(5, Duration::from_secs(3))
      });
      thread::sleep(Duration::from_secs(2));
      let sender = init(remote_addr.clone());
      let res = sender.send(1)
        .send(2)
        .send(3)
        .send(4)
        .send(5);
      assert!(res.is_connected());
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

}
