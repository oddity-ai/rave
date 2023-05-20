pub type Result<T> = std::result::Result<T, Error>;

use crate::packet::Packet;
use crate::packetization::h264::H264PacketizationMode;

#[derive(Debug)]
pub enum Error {
    VersionUnknown { version: usize },
    CsrcCountInvalid { count: usize },
    ExtensionLengthInvalid { len: usize },
    PaddingLengthInvalid { padding_divisor: u8, len: usize },
    NotEnoughData { have: usize, need: usize },
    PacketSizeExceedsMtu { packet: Packet, mtu: usize },
    H264PacketizationModeUnknown { mode: usize },
    H264PacketizationModeUnsupported { mode: H264PacketizationMode },
    H264InvalidNalHeader,
    H264NalDataLengthInvalid { len: usize },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::VersionUnknown { version } => write!(f, "version number unknown: {version}"),
            Error::CsrcCountInvalid { count } => {
                write!(f, "csrc count invalid (overflow): {count}")
            }
            Error::ExtensionLengthInvalid { len: length } => {
                write!(f, "extension length invalid (overflow): {length}")
            }
            Error::PaddingLengthInvalid {
                padding_divisor,
                len,
            } => {
                write!(
                    f,
                    "padding divisor produces invalid padding length (overflow): \
                        {padding_divisor} (to pad {len})",
                )
            }
            Error::NotEnoughData { have, need } => {
                write!(f, "buffer too small: {have} (need {need})")
            }
            Error::PacketSizeExceedsMtu { packet, mtu } => {
                write!(f, "packet size exceeds mtu: {packet:?} > {mtu})")
            }
            Error::H264PacketizationModeUnknown { mode } => {
                write!(f, "h264 packetization mode unknown: {mode})")
            }
            Error::H264PacketizationModeUnsupported { mode } => {
                write!(f, "h264 packetization mode not supported: {mode})")
            }
            Error::H264InvalidNalHeader => {
                write!(f, "expected nal unit but got invalid nal header")
            }
            Error::H264NalDataLengthInvalid { len } => {
                write!(f, "nal data length invalid (overflow): {len}")
            }
        }
    }
}

impl std::error::Error for Error {}
