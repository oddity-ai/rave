use bytes::Bytes;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub version: Version,
    pub padding: bool,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub csrc: Vec<u32>,
    pub extension: Option<Extension>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    Version1,
    Version2,
}

impl TryFrom<usize> for Version {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Version::Version1),
            2 => Ok(Version::Version2),
            _ => Err(Error::VersionUnknown { version: value }),
        }
    }
}

impl Version {
    #[inline]
    pub fn as_number(&self) -> usize {
        match self {
            Version::Version1 => 1,
            Version::Version2 => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    pub profile_identifier: u16,
    pub data: Vec<u32>,
}
