// use core::time;
// use std::{task::ready, thread, time::Duration};

// use self::{state::{error::Result, Ready, Pending}, types::{array::Array, MyResult::*}};

use super::*; 
mod state;

// pub struct Sender {
//   remote_addr: String,
//   // ready_state == None when ready_state is not ready to send
//   ready_state: Option<Ready>,
//   pending_state: Option<Pending>,
//   sending: Option<u8>,
//   delivered: Array<u8>,
// }

// #[derive(Clone, Debug)]
// enum SenderError {
//   RemoteUnavailable,
//   Timeout,
//   SendFailed,
//   IllegalState
// }
// use SenderError::*;
// type MyResult<T> = crate::types::MyResult<T, SenderError>;

// const SLEEP_TIME_SECONDS: u64 = 1;
// const TIMEOUT_SECONDS: u64 = 120;

// pub fn init(remote_addr: String) -> Sender {
//   Sender {
//     remote_addr,
//     ready_state: None,
//     pending_state: None,
//     sending: None,
//     delivered: Array::new(),
//   }
// }
// /// Setup the sender part of the perfect link with the remote address
// /// This function always return an instance of Sender. Blocks until the remote is available.
// // fn connect(remote_addr: String) -> Sender {
// //   let res = try_connect(remote_addr, Duration::from_secs(SLEEP_TIME_SECONDS),
// //      Duration::ZERO, Duration::from_secs(TIMEOUT_SECONDS));
// //   match res {
// //     MyResult::Value(sender) => sender,
// //     MyResult::Error(e) => {
// //       print!("Error setting up sender: {:?}", e);
// //       panic!();
// //     }
// //   }
// // }

// // fn try_connect(remote_addr: String, sleep_time: Duration, time_acc: Duration, timeout: Duration) -> MyResult<Sender> {
// //   if time_acc >= timeout {
// //     return MyResult::Error(Timeout);
// //   }
// //   let res = state::connect(remote_addr);
// //   match res {
// //     Result::Value(ready) => MyResult::Value(Sender {remote_addr, ready_state: ready}),
// //     Result::Error(_) => {
// //       thread::sleep(sleep_time);
// //       try_connect(remote_addr, sleep_time, time_acc + sleep_time, timeout)
// //     }
// //   }
// // }

// // fn handle_send_error(remote_addr: String) -> Sender {
// //   print!("Handling send error unimplemented");
// //   setup(remote_addr)
// // }
// impl Sender {
//   /// Setup the sender part of the perfect link with the remote address
//   /// This function always return an instance of Sender. Blocks until the remote is available.
//   fn connect(self) -> Sender {
//     if self.ready_state.is_some() {
//       return self;
//     }
//     let res = self.try_connect(Duration::from_secs(SLEEP_TIME_SECONDS),
//       Duration::ZERO, Duration::from_secs(TIMEOUT_SECONDS));
//     match res {
//       MyResult::Value(sender) => sender,
//       MyResult::Error(e) => {
//         print!("Error setting up sender: {:?}", e);
//         panic!();
//       }
//     }
//   }

//   fn try_connect(self, sleep_time: Duration, time_acc: Duration, timeout: Duration) -> MyResult<Sender> {
//     if time_acc >= timeout {
//       return MyResult::Error(Timeout);
//     }
//     let res = state::connect(self.remote_addr.clone());
//     match res {
//       Result::Value(ready) => MyResult::Value(self.with_ready_state(ready)),
//       Result::Error(_) => {
//         thread::sleep(sleep_time);
//         self.try_connect(sleep_time, time_acc + sleep_time, timeout)
//       }
//     }
//   }

//   fn send(mut self, data: u8) -> Sender {
//     let s = self.connect_if_required(); // 1. Connect to remote if not connected
//     //   .send_data(data) // 2. Send data
//     //   .await_deliver(data); // 3. Await deliver

//     // 2. Send data
//     let res = s.send_data(data);
//     match res {
//         Result::Value(pending) => s.without_ready_state()
//             .with_pending_state(pending)
//             .await_deliver(data),
//         Result::Error(_) => s,
    
//     // print!("Data delivered: {}\n", data);
//     // s
//     }
//   }

//   fn connect_if_required(self) -> Sender {
//     if self.ready_state.is_none() {
//         return self.connect();
//     }
//     self
//   }

//   fn send_data(mut self, data: u8) -> Result<Pending> {
//     match self.ready_state {
//       Some(ready) => {
//         self.sending = Some(data);
//         let res = ready.send(data);
//         res
//       },
//       None => panic!("Illegal state")
//     }
//   }

//   fn handle_send_error(mut self, e: SenderError) -> Sender {
//     self.without_ready_state();
//     panic!("Handling send error unimplemented");
//   }

//   fn await_deliver(mut self, data: u8) -> Sender {
//     match self.pending_state {
//       Some(pending) => {
//         let res = pending.wait_deliver(Duration::from_secs(TIMEOUT_SECONDS));
//         match res {
//           (Result::Value(ready), true) => {
//             self.delivered.push(data);
//             self.without_sending()
//               .with_ready_state(ready)
//           },
//           (Result::Value(_), false) => {
//             self.send(data)
//           }
//           (Result::Error(e), _) => {
//             self.handle_wait_error()
//           }
//         }
//       },
//       None => panic!("Is not pending")
//     }
//   } 

//   fn handle_wait_error(mut self) -> Sender {
//     print!("Handling wait error unimplemented");
//     self
//   }

//   fn with_ready_state(mut self, ready: Ready) -> Sender {
//     self.ready_state = Some(ready);
//     self
//   }

//   fn with_pending_state(mut self, pending: Pending) -> Sender {
//     self.pending_state = Some(pending);
//     self
//   }

//   fn without_pending_state(mut self) -> Sender {
//     self.pending_state = None;
//     self
//   }

//   fn without_ready_state(mut self) -> Sender {
//     self.ready_state = None;
//     self
//   }

//   fn without_sending(mut self) -> Sender {
//     self.sending = None;
//     self
//   }
// }

// #[cfg(test)]
// mod tests {
//     use std::thread;

//     use crate::types::socket::{ServerSocket, Socket};

//     use super::init;

//     #[test]
//     fn can_connect() {
//         let addr = "localhost:8080";
//         let rj = thread::spawn(move || {
//             recevier_accept(addr.to_string().clone());
//         });
//         init(addr.to_string()).connect();
//     }

//     fn run_receiver(addr: String) {
//         let mut socket = recevier_accept(addr);
//         loop {
//             let v = socket.recv().unwrap();
//             print!("Received data {}\n", v);
//             socket.send(v).unwrap();
//             print!("Sent ACK for data {}\n", v);
//         }
//     }

//     fn recevier_accept(addr: String) -> Socket {
//         let s = ServerSocket::bind(addr)
//             .unwrap()
//             .accept()
//             .unwrap();
//         print!("Accepted incoming connection\n");
//         s
//     }
// }
