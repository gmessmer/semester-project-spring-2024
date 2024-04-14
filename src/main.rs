mod sender;
mod messaging;
mod external;
mod types;

use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::io::{self, Read, Write};
use prusti_contracts::*;

#[extern_spec(std::net::TcpStream)]
#[trusted]
fn connect<A>(addr: A) -> io::Result<TcpStream>
where A: ToSocketAddrs;

const DATA_SIZE: usize = 1;

#[derive(Clone)]
pub struct Link {
    pub src: String,
    pub dst: String,
    pub capacity: usize,
}


// type Registry = LinkedList<Message>;
// struct Sender {
//   state: SenderState,
//   registry: Registry
// }

// impl Sender {
//   fn new(link: Link) -> Self {
//     let state = SenderState::new(link)
//       .init()
//       .connect();
//     Sender {
//       state,
//       registry: LinkedList::new()
//     }
//   }

//   fn try_send(self, data: u8) -> Self {
//     let mut new_state: SenderState = self.state.register_job(data)
//       .send()
//       .waitDeliver()
//       .delivered();
//     self.state = self.state.register_job(data)
//       .send()
//       .waitDeliver()
//       .delivered();
//   }
// }

// struct Receiver {
//   state: ReceiverState,
//   registry: Registry
// }


fn main() {
  
}