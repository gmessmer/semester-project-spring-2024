use std::io::ErrorKind;
use std::time::Duration;
use list::*;
use self::messaging::Packet;

use super::{*};
use super::array::Array;

pub struct Socket {
    stream: TcpStream,
    pub sent: List<u8>,
    pub received: List<u8>,
}

type MyResult<T> = crate::types::MyResult<T, SocketError>;

#[derive(Clone)]
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

impl SocketError {
    pub fn to_string(&self) -> String {
        match self {
            SocketError::Timeout => "Timeout".to_string(),
            SocketError::BufferFull => "BufferFull".to_string(),
            SocketError::SendError => "SendError".to_string(),
            SocketError::RecvError => "RecvError".to_string(),
            SocketError::DestinationUnreachable => "DestinationUnreachable".to_string(),
            SocketError::SetTimeoutFailed => "SetTimeoutFailed".to_string(),
            SocketError::BindError => "BindError".to_string(),
            SocketError::AcceptError => "AcceptError".to_string(),
        }
    }
}

use list::List;
use SocketError::*;
impl Socket {
    #[ensures(result.is_ok() ==> result.unwrap_as_ref().sent.len() == 0)]
    #[ensures(result.is_ok() ==> result.unwrap_as_ref().received.len() == 0)]
    pub fn connect(dest: String) -> MyResult<Socket> {
        let stream = TcpStream::connect(dest);
        match stream {
            Ok(stream) => MyResult::Value(Socket { stream, sent: List::new(), received: List::new()}),
            Err(e) => MyResult::Error(DestinationUnreachable),
        }
    }

    /// Send a packet to the underlying stream.
    /// A value of type `()` is returned if the packet is successfully sent.
    /// Otherwise, an error is returned:
    ///    1. SendError: If the write operation fails.
    #[ensures(result.is_ok() ==> self.sent.contains(pkt.seq))]
    pub fn send_pkt(&mut self, pkt: &Packet) -> MyResult<()> {
        // let res = self.stream.write(&pkt.marshall());
        // match res {
        //     Ok(BYTES_PER_PACKET) => MyResult::Value(()),
        //     _ => MyResult::Error(SendError), 
        // }
        match self.send(&pkt.marshall()) {
            MyResult::Value(_) => {
                self.sent.push(pkt.seq);
                MyResult::Value(())
            },
            MyResult::Error(e) => MyResult::Error(e),
        }
    }

    pub fn send(&mut self, bytes: &[u8]) -> MyResult<()> {
        let res = self.stream.write(bytes);
        match res {
            Ok(n) => 
                if n == bytes.len() {
                    MyResult::Value(())
                } else {
                    MyResult::Error(SendError)
                },
            _ => MyResult::Error(SendError),
        }
    }

    pub fn recv(&mut self, bytes: &mut [u8]) -> MyResult<()> {
        let res = self.stream.read(bytes);
        match res {
            Ok(n) => 
                if n == bytes.len() {
                    MyResult::Value(())
                } else {
                    MyResult::Error(RecvError)
                },
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => MyResult::Error(Timeout),
                ErrorKind::TimedOut => MyResult::Error(Timeout),
                _ => MyResult::Error(RecvError),
            },
            _ => MyResult::Error(RecvError),
        }
    }

    /// Receive a packet from the underlying stream.
    /// A value of type `Packet` is returned if a packet is successfully received.
    /// Otherwise, an error is returned:
    ///     1. Timeout: If the read operation takes longer than the timeout duration.
    ///     2. RecvError: If the read operation fails for any other reason.
    #[ensures(result.is_ok() ==> self.received.contains(result.unwrap_as_ref().seq))]
    pub fn recv_pkt(&mut self) -> MyResult<Packet> {
        let mut buffer = [0; 2];
        let result = self.stream.read(&mut buffer);
        match result {
            Ok(BYTES_PER_PACKET) => {
                let pkt = Packet::unmarshall(buffer);
                self.received.push(pkt.seq);
                MyResult::Value(pkt)
            },
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => MyResult::Error(Timeout),
                ErrorKind::TimedOut => MyResult::Error(Timeout),
                _ => MyResult::Error(RecvError),
            },
            _ => MyResult::Error(RecvError),
        }
    }

    pub fn set_read_timeout(&self, timeout: Duration) -> MyResult<()>{
        let result = self.stream.set_read_timeout(Some(timeout));
        match result {
            Ok(_) => MyResult::Value(()),
            Err(_) => MyResult::Error(SetTimeoutFailed),
        }
    }

    pub fn set_write_timeout(&self, timeout: Duration) -> MyResult<()> {
        let result = self.stream.set_write_timeout(Some(timeout));
        match result {
            Ok(_) => MyResult::Value(()),
            Err(_) => MyResult::Error(SetTimeoutFailed),
        }
    }

}

pub struct ServerSocket {
    pub listener: TcpListener,
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
                let s = Socket { stream, sent: List::new(), received: List::new()};
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

    pub fn set_read_timeout(&mut self, timeout: Duration) {
        self.read_timeout = Some(timeout);
    }

    pub fn set_write_timeout(&mut self, timeout: Duration) {
        self.write_timeout = Some(timeout);
    }

}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::types::{socket::Socket, MyResult, messaging::Packet};

    use super::ServerSocket;

    #[test]
    pub fn test_send_recv() {
        let tr = thread::spawn(|| {
            run_server(1)
        });
        run_client(false);
        tr.join().unwrap();
    }

    #[test]
    pub fn test_recv_timeout() {
        let tr = thread::spawn(|| {
            run_server(2)
        });
        run_client(true);
        tr.join().unwrap();
    }

    pub fn run_server(step: usize) {
        let server_socket = ServerSocket::bind("localhost:8080".to_string());
        let mut s = server_socket.unwrap().accept().unwrap();
        s.set_read_timeout(Duration::from_secs(3));
        for _i in 0..step {
            let mut r = s.recv_pkt();
            match r {
                MyResult::Value(pkt) => {
                    print!("Received: {}\n", pkt.to_string());
                    print!("Server done\n")
                },
                MyResult::Error(e) => print!("Error: {:?}\n", e.to_string()),
            }
        }
        
    }

    pub fn run_client(wait_before_send: bool) {
        let mut client = Socket::connect("localhost:8080".to_string()).unwrap();
        if wait_before_send {
            thread::sleep(Duration::from_secs(5));
        }
        let pkt = Packet::new(1, 2);
        let r = client.send_pkt(&pkt).unwrap();
        print!("Sent: {}\n", pkt.to_string());
        
        let pkt2 = Packet::new(2, 2);
        let r = client.send_pkt(&pkt2).unwrap();
        print!("Sent: {}\n", pkt2.to_string());
    }
}