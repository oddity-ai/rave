pub type Result<T> = std::result::Result<T, Error>;

use crate::packetization::h264::H264PacketizationMode;

#[derive(Debug)]
pub enum Error {
    VersionUnknown { version: usize },
    CsrcCountInvalid { count: usize },
    ExtensionLengthInvalid { length: usize },
    PaddingLengthInvalid { padding_divisor: u8, len: usize },
    NotEnoughData { have: usize, need: usize },
    H264PacketizationModeUnknown { mode: usize },
    H264PacketizationModeUnsupported { mode: H264PacketizationMode },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::VersionUnknown { version } => write!(f, "version number unknown: {version}"),
            Error::CsrcCountInvalid { count } => {
                write!(f, "csrc count invalid (overflow): {count}")
            }
            Error::ExtensionLengthInvalid { length } => {
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
            Error::H264PacketizationModeUnknown { mode } => {
                write!(f, "h264 packetization mode unknown: {mode})")
            }
            Error::H264PacketizationModeUnsupported { mode } => {
                write!(f, "h264 packetization mode not supported: {mode})")
            }
        }
    }
}

impl std::error::Error for Error {}
