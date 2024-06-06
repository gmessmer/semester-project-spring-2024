#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub struct Packet {
    pub seq: u8,
    pub data: u8,
}

use std::{char::MAX, u8};
use prusti_contracts::*;

pub const BYTES_PER_PACKET: usize = 2;
pub const MAX_SEQ: u8 = 255; // u8::MAX (255), using u8::MAX does not work with prusti

// #[pure]
// #[ensures(seq == MAX_SEQ ==> result == 0)]
// #[ensures(seq < MAX_SEQ ==> result == seq + 1)]
// pub fn next_seq(seq: u8) -> u8 {
//     if seq == MAX_SEQ {
//         0
//     } else {
//         seq + 1
//     }
// }

// #[pure]
// #[ensures(seq == 0 ==> result == MAX_SEQ)]
// #[ensures(seq > 0 ==> result == seq - 1)]
// pub fn previous_seq(seq: u8) -> u8 {
//     if seq == 0 {
//         255
//     } else {
//         seq - 1
//     }
// }

pub fn ack_with_seq(seq: u8) -> Packet {
    Packet { seq, data: 0 }
}

impl Packet {
    pub fn new(seq: u8, data: u8) -> Self {
        Packet { seq, data }
    }

    pub fn to_ack(&self) -> Packet {
        Packet { seq: self.seq, data: 0 }
    }

    

    pub fn is_ack_of(&self, pkt: &Packet) -> bool {
        self.data == 0 && self.seq == pkt.seq
    }

    pub fn marshall(&self) -> [u8; 2] {
        [self.seq, self.data]
    }

    pub fn unmarshall(data: [u8; 2]) -> Self {
        Packet { seq: data[0], data: data[1] }
    }

    pub fn seq(&self) -> u8 {
        self.seq
    }

    pub fn data(&self) -> u8 {
        self.data
    }

    pub fn to_string(&self) -> String {
        let mut res = String::from("(");
        let r2 = String::from(self.seq.to_string());
        let r3 = String::from(":");
        let r4 = String::from(self.data.to_string());
        let r5 = String::from(")");
        res.push_str(&r2);
        res.push_str(&r3);
        res.push_str(&r4);
        res.push_str(&r5);
        res

        // format!("(seq:{},data:{})", self.seq, self.data)
    }

    pub fn to_string_verbose(&self) -> String {
        let mut res = String::from("Packet");
        res.push_str(&self.to_string());
        res
    }

}