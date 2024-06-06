use std::thread;

use super::*;
mod state;
mod state0;
use self::state::*;
use crate::types::MyResult::{self, *};

pub struct Receiver {
  
}
#[derive(Clone)]
pub enum Error {
    Failure,
}

type Result<T> = MyResult<T, Error>;



fn handle_connection(listening_state: Listening) -> Result<()> {
    let res = listening_state.recv();
    match res {
        MyResult::Value(deliver) => {
            // print!("RECEIVER: received packet\n");
            let res1 = deliver.deliver();
            match res1 {
                MyResult::Value(listening) => {
                    // print!("RECEIVER: packet delivered\n");
                    handle_connection(listening)
                },
                MyResult::Error(_) => return MyResult::Error(Error::Failure),
            }
        },
        MyResult::Error(_) => return MyResult::Error(Error::Failure),
    }
}

fn accept_incoming(ready_state: &Ready) {
    // print!("RECEIVER: binding succesful, waiting for incoming connections\n");
    loop {
        let res = ready_state.accept();
        match res {
            MyResult::Value(listening) => {
                // print!("RECEIVER: connection established\n");
                thread::spawn(move || {
                    handle_connection(listening);
                });
            },
            MyResult::Error(_) => continue,
        }
    }
}

pub fn serve(src_addr: String) -> Result<()> {
    // print!("RECEIVER: binding to {}\n", src_addr);
    let res = state::bind(src_addr);
    match res {
        MyResult::Value(ready) => {
            accept_incoming(&ready);
            Value(())
        },
        MyResult::Error(_) => return MyResult::Error(Error::Failure),
    }
}

mod tests {
    use std::time::Duration;

    use super::*;
    use crate::types::socket::ServerSocket;
    use super::sender;

    #[test]
    pub fn test_serve() {
        let src_addr = "localhost:8080";
        let tr = thread::spawn(|| {
            thread::sleep(Duration::from_secs(5));
            let res = serve(src_addr.to_string());
            match res {
                MyResult::Value(_) => (),
                MyResult::Error(_) => (),
            }
        });
        // thread::sleep(Duration::from_secs(2));
        let sender = sender::init(src_addr.to_string());
        let res = sender.send(1);
    }

}