use std::convert::TryInto;

use serde::Serialize;

use crate::pb::{self};

pub const MAGIC: [u8; 2] = [0xA5, 0xA5];

#[derive(Debug, Clone)]
pub enum PacketData {
    PROTOBUF( __OPENSOURCE__DELETED__ __OPENSOURCE__DELETED__),
    DATA(Vec<u8>),
    NETWORK(Vec<u8>),
    UNSUPPORT(Vec<u8>),
}

/* ─── 基本枚举 ─────────────────────────────────────────────── */
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PktType {
    Ack = 1,
    SessionConfig = 2,
    Data = 3,
}
impl From<u8> for PktType {
    fn from(b: u8) -> Self {
        match b & 0x0F {
            1 => Self::Ack,
            2 => Self::SessionConfig,
            3 => Self::Data,
            _ => Self::Data, // fallback
        }
    }
}
impl From<PktType> for u8 {
    fn from(t: PktType) -> u8 {
        t as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Pb = 1,
    Mass = 2,
    MassVoice = 3,
    FileSensor = 4,
    FileFitness = 5,
    OTA = 6,
    NetWork = 7,
    Lyra = 8,
    FileResearch = 9,
}
impl From<u8> for Channel {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Pb,
            2 => Self::Mass,
            3 => Self::MassVoice,
            4 => Self::FileSensor,
            5 => Self::FileFitness,
            6 => Self::OTA,
            7 => Self::NetWork,
            8 => Self::Lyra,
            9 => Self::FileResearch,
            _ => Self::Pb,
        }
    }
}
impl From<Channel> for u8 {
    fn from(c: Channel) -> u8 {
        c as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Plain = 1,
    Encrypted = 2,
}

/* ─── 辅助结构 ─────────────────────────────────────────────── */
#[derive(Debug, Clone, Copy)]
pub struct PacketPayload<'a> {
    pub channel: Channel,
    pub opcode: OpCode,
    pub data: &'a [u8],
}

/* ─── 核心：Mi __OPENSOURCE__DELETED__（body = Vec<u8>） ─────────────────── */
#[derive(Debug, Clone, Serialize)]
pub struct Mi __OPENSOURCE__DELETED__ {
    pub pkt_type: PktType,
    pub seq: u8,
    pub body: Vec<u8>,
}

impl Mi __OPENSOURCE__DELETED__ {
    /* -- 解析 ------------------------------------------------ */
    pub fn parse(buf: &[u8]) -> Result<Self, &'static str> {
        if buf.len() < 8 {
            return Err("Incomplete header");
        }
        if &buf[..2] != MAGIC {
            return Err("Magic mismatch");
        }

        let pkt_type = PktType::from(buf[2]);
        let seq = buf[3];
        let len = u16::from_le_bytes(buf[4..6].try_into().unwrap()) as usize;
        let frame_sz = 8 + len;
        if buf.len() < frame_sz {
            return Err("Incomplete frame");
        }

        let crc_given = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let crc_calc = crc16_arc(&buf[8..frame_sz]);
        if crc_given != crc_calc {
            return Err("CRC mismatch");
        }

        Ok(Self {
            pkt_type,
            seq,
            body: buf[8..frame_sz].to_vec(), // ← 拷贝进 Vec
        })
    }

    /* -- 批量解析 ------------------------------------------- */
    pub fn parse_all(buf: &[u8]) -> Result<Vec<Self>, &'static str> {
        let mut packets = Vec::new();
        let mut idx = 0;

        while idx < buf.len() {
            if buf[idx..].starts_with(&MAGIC) {
                // ok
            } else if let Some(rel) = buf[idx + 1..].iter().position(|&b| b == MAGIC[0]) {
                idx += rel + 1;
                continue;
            } else {
                return Err("Magic not found");
            }

            let pkt = Self::parse(&buf[idx..])?;
            let frame_sz = 8 + pkt.body.len();
            packets.push(pkt);
            idx += frame_sz;
        }
        Ok(packets)
    }

    /* -- 解析 Data 帧字段 ----------------------------------- */
    pub fn data_fields(&self) -> Option<PacketPayload> {
        if self.pkt_type != PktType::Data || self.body.len() < 2 {
            return None;
        }
        let ch = Channel::from(self.body[0]);
        let op = match self.body[1] {
            1 => OpCode::Plain,
            2 => OpCode::Encrypted,
            _ => return None,
        };
        Some(PacketPayload {
            channel: ch,
            opcode: op,
            data: &self.body[2..],
        })
    }

    /* -- 序列化 --------------------------------------------- */
    pub fn encode(&self) -> Vec<u8> {
        let len = self.body.len() as u16;
        let crc = crc16_arc(&self.body);

        let mut out = Vec::with_capacity(8 + self.body.len());
        out.extend_from_slice(&MAGIC);
        out.push(self.pkt_type.into());
        out.push(self.seq);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&self.body);
        out
    }

    /* -- 辅助构造器：Data 帧 -------------------------------- */
    pub fn new_data(seq: u8, channel: Channel, op: OpCode, payload: &[u8]) -> Self {
        let mut inner = Vec::with_capacity(2 + payload.len());
        inner.push(channel.into()); // 0 = Channel
        inner.push(op as u8); // 1 = OpCode
        inner.extend_from_slice(payload);

        Self {
            pkt_type: PktType::Data,
            seq,
            body: inner,
        }
    }
}

/* ─── CRC-16/ARC 实现 ──────────────────────────────────────── */
fn crc16_arc(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            crc = if (crc & 0x0001) != 0 {
                (crc >> 1) ^ 0xA001 // 0x8005 reflected
            } else {
                crc >> 1
            };
        }
    }
    crc
}
