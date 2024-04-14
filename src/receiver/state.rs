use super::*;

pub enum ReceiverState {
    Init {link: Link},
    Bind {link: Link, src_addr: String},
    Ready {link: Link, listener: TcpListener},
    Listening {link: Link, stream: TcpStream},
    Deliver {link: Link, stream: TcpStream, msg: Message},
    IllegalState,
    Error,
}

impl ReceiverState {

  #[ensures(matches!(result, ReceiverState::Init {link}))]
  fn new(link: Link) -> Self {
    ReceiverState::Init {link}
  }

  predicate! {
    fn is_init(&self) -> bool {
      match self {
        ReceiverState::Init {link} => true,
        _ => false
      }
    }
  }

  predicate! {
    fn is_bind(&self) -> bool {
      match self {
        ReceiverState::Bind {link, src_addr} => true,
        _ => false
      }
    }
  }

  predicate! {
    fn is_listening(&self) -> bool {
      match self {
        ReceiverState::Listening {link, stream} => true,
        _ => false
      }
    }
  }

  #[requires(self.is_init())]
  fn init(self) -> ReceiverState {
    match self {
      ReceiverState::Init {link} => {
        let src_addr = link.src.clone();
        ReceiverState::Bind {link, src_addr}
      },
      _ => ReceiverState::IllegalState
    }
  }

  #[requires(self.is_bind())]
  fn bind(self) -> ReceiverState {
    match self {
      ReceiverState::Bind {link, src_addr} => {
        let listener = TcpListener::bind(src_addr);
        match listener {
          Ok(listener) => ReceiverState::Ready {link, listener},
          Err(_) => ReceiverState::Error
        }
      },
      _ => ReceiverState::IllegalState
    }
  }

  #[requires(matches!(self, ReceiverState::Ready {link, listener}))]
  fn accept(self) -> ReceiverState {
    match self {
      ReceiverState::Ready {link, listener} => {
        match listener.accept() {
          Ok((stream, _)) => ReceiverState::Listening { link, stream},
          Err(_) => ReceiverState::Error
        }
      },
      _ => ReceiverState::IllegalState
    }
  }

  #[requires(matches!(self, ReceiverState::Listening {link, stream}))]
  fn listen(self) -> ReceiverState {
    match self {
      ReceiverState::Listening {link, mut stream} => {
        let mut buf = [0; 3];
        let n = stream.read(&mut buf);
        match n {
          Ok(3) => {
            match Message::unmarshall(&buf) {
              Some(msg) => ReceiverState::Deliver {link, stream, msg},
              None => ReceiverState::Error
            }
          },
          _ => ReceiverState::Error
        }
      },
      _ => ReceiverState::IllegalState
    }
  }

  #[requires(matches!(self, ReceiverState::Deliver {link, stream, msg}))]
  fn deliver(self) -> ReceiverState {
    match self {
      ReceiverState::Deliver {link, mut stream, msg} => {
        let ack = Message::Ack {id: msg.id()};
        let res = stream.write(&ack.marshall());
        match res {
          Ok(2) => ReceiverState::Listening {link, stream},
          _ => ReceiverState::Error
        }
      },
      _ => ReceiverState::IllegalState
    }
  }
}