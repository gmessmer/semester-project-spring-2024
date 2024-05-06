use std::time::Duration;

use self::messaging::Message;

use super::{*};
use super::array::Array;

pub struct Socket {
    stream: TcpStream,
    sent: Array<u8>,
    received: Array<u8>,
}

type MyResult<T> = crate::types::MyResult<T, SocketError>;

#[derive(Clone, Debug)]
pub enum SocketError {
    Timeout,
    BufferFull,
    SendError,
    RecvError,
    DestinationUnreachable,
    SetTimeoutFailed,
    BindError,
    AcceptError
}

use SocketError::*;
impl Socket {
    #[ensures(result.is_ok() ==> result.unwrap().sent.len() == 0)]
    #[ensures(result.is_ok() ==> result.unwrap().received.len() == 0)]
    pub fn connect(dest: String) -> MyResult<Socket> {
        let stream = TcpStream::connect(dest);
        match stream {
            Ok(stream) => MyResult::Value(Socket { stream, sent: Array::new(), received: Array::new()}),
            Err(e) => MyResult::Error(DestinationUnreachable),
        }
    }

    pub fn send_msg(&mut self, pkt: Packet) -> MyResult<()> {
        let res = self.stream.write(&pkt.marshall());
        match res {
            Ok(BYTES_PER_PACKET) => MyResult::Value(()),
            _ => MyResult::Error(SendError), 
        }
    }

    pub fn recv_msg(&mut self, pkt: Packet) -> MyResult<Packet> {
        let mut buffer = [0; 2];
        let result = self.stream.read(&mut buffer);
        match result {
            Ok(n) => {
                if n == 0 {
                    MyResult::Error(Timeout)
                } else if n == BYTES_PER_PACKET {
                    let pkt = Packet::unmarshall(buffer);
                    MyResult::Value(pkt)
                } else {
                    MyResult::Error(RecvError)
                }
            },
            Err(e) => MyResult::Error(RecvError),
        }
    }

    #[ensures(result.is_ok() ==> self.sent.len() == old(self.sent.len()) + 1)]
    #[ensures(self.received.len() == old(self.received.len()))]
    #[ensures(!result.is_ok() ==> self.sent.len() == old(self.sent.len()))]
    pub fn send(&mut self, data: u8) -> MyResult<usize> {
        let result = self.stream.write(&[data]);
        match result {
            Ok(n) => {
                match self.sent.push(data) {
                    MyResult::Value(_) => MyResult::Value(n),
                    MyResult::Error(_) => MyResult::Error(BufferFull),
                }
            },
            Err(e) => MyResult::Error(SendError),
        }
    }

    /// Receive a single byte
    #[ensures(result.is_ok() ==> self.received.len() == old(self.received.len()) + 1)]
    // #[ensures(result.is_ok() ==> self.received.last().unwrap() == result.unwrap())]
    // ensure contains??
    pub fn recv(&mut self) -> MyResult<u8> {
        let mut buffer = [0; 1];
        let result = self.stream.read(&mut buffer);
        match result {
            Ok(n) => {
                if n == 0 {
                    return MyResult::Error(Timeout);
                }
                match self.received.push(buffer[0]) {
                    MyResult::Value(_) => MyResult::Value(buffer[0]),
                    MyResult::Error(_) => MyResult::Error(BufferFull),
                }
            },
            Err(e) => MyResult::Error(RecvError),
        }
    }

    #[pure]
    pub fn nsent(&self) -> usize {
        self.sent.len()
    }

    #[pure]
    pub fn nrecv(&self) -> usize {
        self.received.len()
    }

    pub fn set_read_timeout(&self, timeout: Duration) -> MyResult<()>{
        let result = self.stream.set_read_timeout(Some(timeout));
        match result {
            Ok(_) => MyResult::Value(()),
            Err(e) => MyResult::Error(SetTimeoutFailed),
        }
    }

    pub fn set_write_timeout(&self, timeout: Duration) -> MyResult<()> {
        let result = self.stream.set_write_timeout(Some(timeout));
        match result {
            Ok(_) => MyResult::Value(()),
            Err(e) => MyResult::Error(SetTimeoutFailed),
        }
    }

}

pub struct ServerSocket {
    listener: TcpListener,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl ServerSocket {
    
    pub fn bind(src: String) -> MyResult<ServerSocket> {
        let listener = TcpListener::bind(src);
        match listener {
            Ok(listener) => MyResult::Value(ServerSocket { listener , read_timeout: None, write_timeout: None}),
            Err(e) => MyResult::Error(BindError),
        }
    }


    pub fn accept(&self) -> MyResult<Socket> {
        let stream = self.listener.accept();
        match stream {
            Ok((stream, _)) => {
                let s = Socket { stream, sent: Array::new(), received: Array::new()};
                if self.read_timeout.is_some() {
                    let err = s.set_read_timeout(self.read_timeout.unwrap());
                    if err.is_err() {
                        return MyResult::Error(err.unwrap_err().clone());
                    }
                }
                if self.write_timeout.is_some() {
                    let err = s.set_write_timeout(self.write_timeout.unwrap());
                    if err.is_err() {
                        return MyResult::Error(err.unwrap_err().clone());
                    }
                }
                MyResult::Value(s)
            },
            Err(e) => MyResult::Error(AcceptError),
        }
    }

    pub fn set_read_timeout(&mut self, timeout: Duration) -> &ServerSocket {
        self.read_timeout = Some(timeout);
        self
    }

    pub fn set_write_timeout(&mut self, timeout: Duration) -> &ServerSocket{
        self.write_timeout = Some(timeout);
        self
    }

}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::types::{socket::Socket, MyResult};

    use super::ServerSocket;

    #[test]
    pub fn test_timeout() {
        let tr = thread::spawn(|| {
            run_server()
        });
        Socket::connect("localhost:8080".to_string()).unwrap()
            .send(12).unwrap();
        tr.join().unwrap();
    }

    pub fn run_server() {
        let mut s = ServerSocket::bind("localhost:8080".to_string()).unwrap()
            .accept().unwrap();
        thread::sleep(Duration::from_secs(2));
        s.set_read_timeout(Duration::from_secs(3));
        let r = s.recv();
        match r {
            MyResult::MyResult::Value(v) => {
                print!("Received: {}\n", v);
                print!("Server done")
            },
            MyResult::MyResult::Error(e) => print!("Error: {:?}\n", e),
        }
        let r = s.recv();
        match r {
            MyResult::MyResult::Value(v) => {
                print!("Received: {}\n", v);
                print!("Server done\n")
            },
            MyResult::MyResult::Error(e) => print!("Error: {:?}\n", e),
        }
        
    }

    pub fn run_client() {
        let mut client = Socket::connect("localhost:8080".to_string()).unwrap();
        // let r = client.send(1).unwrap();
        // print!("Sent: {}\n", r);
        // print!("Client done");
    }
}