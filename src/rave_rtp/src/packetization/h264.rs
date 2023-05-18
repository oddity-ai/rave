use crate::packet::Packet;

use bytes::Bytes;

type Result<T> = std::result::Result<T, ()>; // TODO

// TODO: derives
pub enum H264PacketizationMode {
    SingleNalUnit,
    NonInterleavedMode,
    InterleavedMode,
}

impl TryFrom<usize> for H264PacketizationMode {
    type Error = (); // TODO: dummy

    fn try_from(mode: usize) -> Result<Self> {
        match mode {
            0 => Ok(H264PacketizationMode::SingleNalUnit),
            1 => Ok(H264PacketizationMode::NonInterleavedMode),
            2 => Ok(H264PacketizationMode::InterleavedMode),
            _ => Err(todo!()),
        }
    }
}

// TODO: derives
pub struct H264Packetizer {
    inner: Box<dyn H264Packetize>,
}

impl H264Packetizer {
    pub fn from_packetization_mode(mode: H264PacketizationMode, mtu: Option<usize>) -> Result<Self> {
        Ok(Self {
            inner: match mode {
                H264PacketizationMode::SingleNalUnit => Box::new(H264PacketizerMode0::new(mtu)),
                H264PacketizationMode::NonInterleavedMode => Box::new(H264PacketizerMode1::new(mtu)),
                H264PacketizationMode::InterleavedMode => todo!(), // TODO: not supported error
            }
        })
    }

    pub fn packetize(nalus: &[Bytes]) -> Vec<Packet> {
        todo!()
    }
}

pub trait H264Packetize {
    /// TODO: mention in docs that all passed h264 nalus MUST be Single-Time (for STAP)
    fn packetize(&mut self, nalus: &[Bytes]) -> Vec<Packet>;
}

// TODO: derives
pub struct H264PacketizerMode0 {
    mtu: Option<usize>, // TODO: return error if payload larger than MTU
}

impl H264PacketizerMode0 {
    pub fn new(mtu: Option<usize>) -> Self {
        Self { mtu, }
    }
}

impl H264Packetize for H264PacketizerMode0 {
    fn packetize(&mut self, nalus: &[Bytes]) -> Vec<Packet> {
        todo!()
    }
}

// TODO: derives
pub struct H264PacketizerMode1 {
    mtu: Option<usize>, // TODO: fragment if payload higher than MTU
}

impl H264PacketizerMode1 {
    pub fn new(mtu: Option<usize>) -> Self {
        Self { mtu, }
    }
}

impl H264Packetize for H264PacketizerMode1 {
    fn packetize(&mut self, nalus: &[Bytes]) -> Vec<Packet> {
        todo!()
    }
}

// TODO: derives
pub struct H264Depacketizer {
    // TODO
}

impl H264Depacketizer {
    // TODO: this holds state like parameter sets as well

    // TODO: docs: in = payload, out = zero or more NALUs
    pub fn depacketize(&mut self, packet: &Packet) -> Vec<Bytes> {
        todo!()
    }
}