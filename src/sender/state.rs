use self::messaging::Message;

use super::*;

pub enum SenderState {
    Init {link: Link},
    Connect {link: Link, remote_addr: String},
    Ready {link: Link, stream: TcpStream},
    Sending {link: Link, stream: TcpStream, data: u8},
    Pending {link: Link, stream: TcpStream, data: u8},
    Delivered {link: Link, stream: TcpStream, data: u8},
    Error, 
    IllegalState,
    SendError
}

impl SenderState {

  #[ensures(matches!(result, SenderState::Init {link}))]
  fn new(link: Link) -> Self {
    SenderState::Init {link}
  }

  predicate! {
    fn is_init(&self) -> bool {
      match self {
        SenderState::Init {link} => true,
        _ => false
      }
    }
  }
  predicate! {
    fn is_connect(&self) -> bool {
      match self {
        SenderState::Connect {link, remote_addr} => true,
        _ => false
      }
    }
  }

  predicate!{
    fn is_ready(&self) -> bool {
      match self {
        SenderState::Ready {link, stream} => true,
        _ => false
      }
    }
  }

  predicate!{
    fn has_capacity(&self, data_size: usize) -> bool {
      match self {
        SenderState::Ready {link, stream} => link.capacity >= data_size,
        _ => false
      }
    }
  }
    

  #[requires(self.is_init())]
  fn init(self) -> SenderState {
    match self {
      SenderState::Init {link} => {
        let remote_addr = link.dst.clone();
        SenderState::Connect {link, remote_addr}
      },
      _ => SenderState::IllegalState
    }
  }

  #[requires(self.is_connect())]
  fn connect(self) -> SenderState {
    match self {
      SenderState::Connect {link, remote_addr} => {
        let stream = TcpStream::connect(remote_addr);
        match stream {
          Ok(stream) => SenderState::Ready {link: link, stream},
          Err(_) => SenderState::Error
        }
      }
      _ => SenderState::IllegalState
    }
  }

  #[requires(matches!(self, SenderState::Ready {link, stream}))]
  #[requires(self.has_capacity(DATA_SIZE))]
  #[ensures(matches!(result, SenderState::Sending {link, stream, data}))]
  fn register_job(self, data: u8) -> SenderState {
    match self {
      SenderState::Ready {link, stream} => {
        SenderState::Sending {link, stream, data: data}
      },
      _ => SenderState::IllegalState
    }
  }


  #[requires(matches!(self, SenderState::Sending {link, stream, data}))]
  #[ensures(matches!(&result, SenderState::Pending {link, stream, data}) || 
      matches!(result, SenderState::SendError))]
  fn send(self) -> SenderState {
    match self {
      SenderState::Sending {link, mut stream, data} => {
        let res = stream.write(&[data]);
        match res {
          Ok(bs) => {
            if bs == DATA_SIZE {
              SenderState::Pending {link, stream, data}
            } else {
              SenderState::SendError
            }
          },
          Err(_) => SenderState::SendError
        }
      }
      _ => SenderState::IllegalState
    }
  }


  #[requires(matches!(self, SenderState::Pending {link, stream, data}))]
  fn waitDeliver(self) -> SenderState {
    match self {
      SenderState::Pending {link, mut stream, data} => {
        let mut buf = [0; 2];
        let n = stream.read(&mut buf);
        // add and handle timeout
        match n {
          Ok(2) => {
            let msg = Message::unmarshall(&buf);
            SenderState::Delivered {link, stream, data}
          },
          _ => SenderState::Pending {link, stream, data}
        }
      },
      _ => SenderState::IllegalState
    }
  }

  #[requires(matches!(self, SenderState::Delivered {link, stream, data}))]
  #[ensures(matches!(result, SenderState::Ready {link, stream}))]
  fn delivered(self) -> SenderState {
    match self {
      SenderState::Delivered {link, stream, ..} => {
        SenderState::Ready {link, stream}
      },
      _ => SenderState::IllegalState
    }
  }

}