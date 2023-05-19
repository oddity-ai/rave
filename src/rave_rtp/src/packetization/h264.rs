use crate::error::Error;
use crate::packet::Packet;

use bytes::Bytes;

type Result<T> = std::result::Result<T, Error>;

pub struct H264Packetizer {
    inner: Box<dyn H264Packetize>,
}

impl H264Packetizer {
    pub fn from_packetization_mode(
        mode: H264PacketizationMode,
        mtu: Option<usize>,
    ) -> Result<Self> {
        Ok(Self {
            inner: match mode {
                H264PacketizationMode::SingleNalUnit => Box::new(H264PacketizerMode0::new(mtu)),
                H264PacketizationMode::NonInterleavedMode => {
                    Box::new(H264PacketizerMode1::new(mtu))
                }
                H264PacketizationMode::InterleavedMode => {
                    return Err(Error::H264PacketizationModeUnsupported { mode })
                }
            },
        })
    }

    pub fn packetize(nalus: &[Bytes]) -> Vec<Packet> {
        todo!()
    }
}

pub trait H264Packetize {
    /// Packetize one or more H264 packets.
    ///
    /// # Correctness
    ///
    /// The passed data must contain one or more wholly contained NAL units. If more than one NAL unit
    /// is contained in the data, all NAL units **must** have the same original timestamp, since they
    /// might be packed into a Single-Time Aggregation Packet.
    ///
    /// # Arguments
    ///
    /// * `data` - Data containing one or more H264 packets (serialzied NAL units).
    ///
    /// # Return value
    ///
    /// Zero or more packets.
    ///
    /// No packets may be returned even if valid data was passed. More than one packet may be produced
    /// if the data is fragmented over multiple packets to fit within the configured MTU.
    fn packetize(&mut self, data: &Bytes) -> Vec<Packet>;
}

#[derive(Debug)]
pub struct H264PacketizerMode0 {
    mtu: Option<usize>, // TODO: return error if payload larger than MTU
}

impl H264PacketizerMode0 {
    pub fn new(mtu: Option<usize>) -> Self {
        Self { mtu }
    }
}

impl H264Packetize for H264PacketizerMode0 {
    fn packetize(&mut self, data: &Bytes) -> Vec<Packet> {
        todo!()
    }
}

#[derive(Debug)]
pub struct H264PacketizerMode1 {
    mtu: Option<usize>, // TODO: fragment if payload higher than MTU
}

impl H264PacketizerMode1 {
    pub fn new(mtu: Option<usize>) -> Self {
        Self { mtu }
    }
}

impl H264Packetize for H264PacketizerMode1 {
    fn packetize(&mut self, data: &Bytes) -> Vec<Packet> {
        todo!()
    }
}

#[derive(Debug)]
pub struct H264Depacketizer {
    // TODO
}

impl H264Depacketizer {
    // TODO: this holds state like parameter sets as well

    // TODO: docs: in = payload, out = zero or more NALUs

    /// Depacketize RTP packets and convert back to raw H264 NAL units that can be passed to a
    /// decoder.
    ///
    /// This function will reconstruct fragmented NALUs, as well as split aggregation packets back
    /// into separate H264 NAL units.
    ///
    /// # Arguments
    ///
    /// * `packet` - RTP Packet to depacketize.
    ///
    /// # Return value
    ///
    /// Zero or more H264 packets ready for decoding.
    ///
    /// No NAL units may be produced if the packet contains part of a fragmented unit. More packets
    /// may be produced if the RTP packet payload is an aggregation packet (STAP or MTAP).
    pub fn depacketize(&mut self, packet: &Packet) -> Vec<Bytes> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264PacketizationMode {
    SingleNalUnit,
    NonInterleavedMode,
    InterleavedMode,
}

impl TryFrom<usize> for H264PacketizationMode {
    type Error = Error; // TODO: dummy

    fn try_from(mode: usize) -> Result<Self> {
        match mode {
            0 => Ok(H264PacketizationMode::SingleNalUnit),
            1 => Ok(H264PacketizationMode::NonInterleavedMode),
            2 => Ok(H264PacketizationMode::InterleavedMode),
            _ => Err(Error::H264PacketizationModeUnknown { mode }),
        }
    }
}

impl std::fmt::Display for H264PacketizationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            H264PacketizationMode::SingleNalUnit => write!(f, "single nal unit"),
            H264PacketizationMode::NonInterleavedMode => write!(f, "non-interleaved mode"),
            H264PacketizationMode::InterleavedMode => write!(f, "interleaved mode"),
        }
    }
}
