use super::*;

#[derive(Clone)]
pub enum Message {
    Data {id: u8, data: u8},
    Ack {id: u8},
}

impl Message {

  pub fn marshall(self) -> Vec<u8> {
    match self {
      Message::Data {id, data} => {
        let mut buf = Vec::new();
        buf.push(0x0);
        buf.push(id);
        buf.push(data);
        buf
      },
      Message::Ack {id} => {
        let mut buf: Vec<u8> = Vec::new();
        buf.push(0x1);
        buf.push(id);
        buf
      }
    }
  }


  pub fn unmarshall(buf: &[u8]) -> Option<Message> {
    if buf.len() < 2 {
      return None;
    }
    let id = buf[1];
    let code = buf[0];
    match code {
      0x0 => {
        if buf.len() < 3 {
          return None;
        }
        Some(Message::Data {id, data: buf[2]})
      },
      0x1 => Some(Message::Ack {id}),
      _ => None
    }
  }

  fn id(&self) -> u8 {
    match self {
      Message::Data {id, ..} => *id,
      Message::Ack {id} => *id
    }
  }
}