use types::socket::Socket;
use types::{list::List, messaging::Packet};

use super::*;

#[derive(Clone)]
enum State {
    Listen, 
    Acknowledge,
    Crash,
}

#[derive(Clone)]
pub enum Error {
    Fatal, 
    Timeout,
    OutOfOrderPacket,
}

type Result<T> = MyResult<T, Error>;

pub struct ReceiverState {
    state: State,
    socket: Socket,
    seq: u8,
    acked: List<u8>,
}

pub fn new(socket: Socket, initial_seq: u8) -> ReceiverState {
    ReceiverState {
        state: State::Listen,
        socket,
        seq: 0,
        acked: List::new(),
    }
}

#[pure]
#[ensures(result == (pkt.seq == seq || pkt.seq == seq - 1))]
fn is_valid_pkt(pkt: &Packet, seq: u8) -> bool {
    pkt.seq == seq || pkt.seq == seq - 1
}

impl ReceiverState {
    #[requires(self.state === State::Listen)]
    #[ensures(result.is_ok() ==> self.state === State::Acknowledge && is_valid_pkt(result.unwrap_as_ref(), old(self).seq))]
    #[ensures(result === Error(Error::OutOfOrderPacket) ==> self.state === State::Listen)]
    #[ensures(result === Error(Error::Fatal) ==> self.state === State::Crash)]
    pub fn recv(&mut self) -> Result<Packet> {
        match self.state {
            State::Listen => {
                match self.socket.recv_pkt() {
                    Value(pkt) => {
                        if is_valid_pkt(&pkt, self.seq) {
                            self.seq = pkt.seq;
                            self.state = State::Acknowledge;
                            Value(pkt)
                        } else {
                            Error(Error::OutOfOrderPacket)
                        }                        
                    },
                    Error(_) => {
                        self.state = State::Crash;
                        Error(Error::Fatal)
                    },
                }
            },
            _ => unreachable!(),
        }
    }

    #[requires(self.state === State::Acknowledge)]
    #[requires((pkt.seq === self.seq || pkt.seq === self.seq - 1 ))]
    #[ensures(result.is_ok() ==> self.state === State::Listen)]
    #[ensures(result.is_ok() ==> self.seq === pkt.seq + 1)]
    #[ensures(result.is_ok() ==> self.acked.is_union(old(&self.acked), pkt.seq))]
    #[ensures(result === Error(Error::Fatal) ==> self.state === State::Crash)]
    pub fn ack(&mut self, pkt: Packet) -> Result<()> {
        match self.state {
            State::Acknowledge => {
                if pkt.seq == self.seq || pkt.seq == self.seq - 1 {
                    let ack_pkt = Packet { seq: pkt.seq + 1, data: 0 };
                    match self.socket.send_pkt(&ack_pkt) {
                        Value(_) => {
                            self.seq = ack_pkt.seq;
                            self.acked.push(pkt.seq);
                            self.state = State::Listen;
                            Value(())
                        },
                        Error(_) => {
                            self.state = State::Crash;
                            return Error(Error::Fatal);
                        },
                    }
                } else {
                    unreachable!();
                }
            },
            _ => unreachable!(),
        }
    }

    #[requires(self.state === State::Crash)]
    #[ensures(self.state === State::Listen)]
    pub fn recover(&mut self, socket: Socket) {
        self.state = State::Listen;
        self.socket = socket;
    }

}