use socket::Socket;
use types::{list::List, messaging::Packet};

use super::*;

#[derive(Clone)]
enum State {
    Ready, 
    Pending,
    Crashed,
}

#[derive(Clone)]
pub enum Error {
    Fatal, 
    Timeout,
}

type Result<T> = MyResult<T, Error>;

pub struct SenderState {
    state: State,
    socket: Socket,
    seq: u8,
    sent: List<u8>,
}

pub fn new(socket: Socket, initial_seq: u8) -> SenderState {
    SenderState {
        state: State::Ready,
        socket,
        seq: 0,
        sent: List::new(),
    }
}

impl SenderState {
    // predicate!(
    //     fn sent_UDP(&self, data: u8) -> bool {
    //         self.sent.contains(data)
    //     }
    // );
    #[requires(self.state === State::Ready)]
    #[ensures(result.is_ok() ==> self.state === State::Pending)]
    #[ensures(result.is_ok() ==> self.sent.is_union(old(&self.sent), data))]
    pub fn send(&mut self, data: u8) -> Result<()> {
        match self.state {
            State::Ready => {
                let pkt = Packet { seq: self.seq, data };
                match self.socket.send_pkt(&pkt) {
                    Value(()) => {
                        self.state = State::Pending;
                        self.sent.push(data);
                        Value(())
                    },
                    Error(_) => Error(Error::Fatal),
                
                }
            },
            _ => unreachable!(),
        }
    }

    
    #[requires(self.state === State::Pending)]
    #[ensures(result.is_ok() ==> self.state === State::Ready)]
    #[ensures(result.is_ok() ==> self.seq === old(self.seq) + 1)]
    #[ensures(result === Error(Error::Timeout) ==> self.state === State::Ready)]
    #[ensures(result === Error(Error::Fatal) ==> self.state === State::Crashed)]
    pub fn wait_deliver(&mut self, timeout: Duration) -> Result<()> {
        match self.state {
            State::Pending => {
                let t0 = std::time::Instant::now();
                let r = self.socket.set_read_timeout(timeout);
                if r.is_err() {
                    self.state = State::Crashed;
                    return Error(Error::Fatal);
                }
                match self.socket.recv_pkt() {
                    Value(pkt) => {
                        if pkt.seq == self.seq {
                            self.seq += 1;
                            self.state = State::Ready;
                            Value(())
                        } else {
                            let delta = std::time::Instant::now().duration_since(t0);
                            if timeout > delta {
                                self.wait_deliver(timeout - delta)
                            } else {
                                self.state = State::Ready;
                                Error(Error::Timeout)
                            }
                        }
                    },
                    Error(_) => {
                        self.state = State::Crashed;
                        Error(Error::Fatal)
                    },
                }
            },
            _ => unreachable!(),
        }
    }

    #[requires(self.state === State::Crashed)]
    #[ensures(self.state === State::Ready)]
    pub fn recover(&mut self, socket: Socket) {
        self.state = State::Ready;
        self.socket = socket;
    }
}