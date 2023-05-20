use bytes::Bytes;

use crate::error::{Error, Result};
use crate::packet::{Header, Packet, Version};
use crate::serialize::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketizationParameters {
    pub payload_type: u8,
    pub ssrc: u32,
    pub csrc: Vec<u32>,
    pub mtu: Option<usize>,
}

#[derive(Debug)]
pub struct Packetizer {
    header: Header,
    header_serialized_len: usize,
    sequence_number: u16,
    mtu: Option<usize>,
}

impl Packetizer {
    pub fn new(payload_type: u8, ssrc: u32, csrc: Vec<u32>, mtu: Option<usize>) -> Self {
        let header = Header {
            version: Version::Version2,
            padding: false,
            marker: false,
            payload_type,
            sequence_number: 0,
            timestamp: 0,
            ssrc,
            csrc,
            extension: None,
        };
        let header_serialized_len = header.serialized_len();
        Self {
            header,
            header_serialized_len,
            sequence_number: rand::random::<u16>(),
            mtu,
        }
    }

    pub fn from_packetization_parameters(
        packetization_parameters: PacketizationParameters,
    ) -> Self {
        Self::new(
            packetization_parameters.payload_type,
            packetization_parameters.ssrc,
            packetization_parameters.csrc,
            packetization_parameters.mtu,
        )
    }

    pub fn packetize(&mut self, payload: Bytes, timestamp: u32, marker: bool) -> Result<Packet> {
        let mut header = self.header.clone();
        header.marker = marker;
        header.sequence_number = self.next_sequence_number();
        header.timestamp = timestamp;

        let packet = Packet::new(header, payload);

        if let Some(mtu) = self.mtu {
            if packet.serialized_len() > mtu {
                return Err(Error::PacketSizeExceedsMtu { packet, mtu });
            }
        }

        Ok(packet)
    }

    #[inline]
    pub fn header_serialized_len(&self) -> usize {
        self.header_serialized_len
    }

    #[inline]
    fn next_sequence_number(&mut self) -> u16 {
        let sequence_number = self.sequence_number;
        self.sequence_number = self.sequence_number.wrapping_add(1);
        sequence_number
    }
}
