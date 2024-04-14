use super::*;
use super::MyResult::*;
use super::array::Array;
pub struct Socket {
    stream: TcpStream,
    sent: Array<u8>,
    received: Array<u8>,
}


impl Socket {
    #[ensures(result.is_ok() ==> result.unwrap().sent.len() == 0)]
    #[ensures(result.is_ok() ==> result.unwrap().received.len() == 0)]
    pub fn connect(dest: String) -> MyResult<Socket> {
        let stream = TcpStream::connect(dest);
        match stream {
            Ok(stream) => Value(Socket { stream, sent: Array::new(), received: Array::new()}),
            Err(e) => Error("Failed to connect".to_string() + &e.to_string()),
        }
    }

    #[ensures(result.is_ok() ==> self.sent.len() == old(self.sent.len()) + 1)]
    #[ensures(self.received.len() == old(self.received.len()))]
    #[ensures(!result.is_ok() ==> self.sent.len() == old(self.sent.len()))]
    pub fn send(&mut self, data: u8) -> MyResult<()> {
        let result = self.stream.write(&[data]);
        match result {
            Ok(_) => {
                match self.sent.push(data) {
                    Value(_) => Value(()),
                    Error(_) => Error("Failed to send: buffer is full".to_string()),
                }
            },
            Err(e) => Error("Failed to send".to_string() + &e.to_string()),
        }
    }

    #[ensures(result.is_ok() ==> self.received.len() == old(self.received.len()) + 1)]
    // #[ensures(result.is_ok() ==> self.received.last().unwrap() == result.unwrap())]
    // ensure contains??
    pub fn recv(&mut self) -> MyResult<u8> {
        let mut buffer = [0; 1];
        let result = self.stream.read(&mut buffer);
        match result {
            Ok(_) => {
                match self.received.push(buffer[0]) {
                    Value(_) => Value(buffer[0]),
                    Error(_) => Error("Failed to receive: buffer is full".to_string()),
                }
            },
            Err(e) => Error("Failed to receive".to_string() + &e.to_string()),
        }
    }
}

pub struct ServerSocket {
    listener: TcpListener,
}

impl ServerSocket {
    pub fn bind(src: String) -> MyResult<ServerSocket> {
        let listener = TcpListener::bind(src);
        match listener {
            Ok(listener) => Value(ServerSocket { listener }),
            Err(e) => Error("Failed to bind".to_string() + &e.to_string()),
        }
    }


    pub fn accept(&self) -> MyResult<Socket> {
        let stream = self.listener.accept();
        match stream {
            Ok((stream, _)) => Value(Socket { stream, sent: Array::new(), received: Array::new()}),
            Err(e) => Error("Failed to accept".to_string() + &e.to_string()),
        }
    }

}

