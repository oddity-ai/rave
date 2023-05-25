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
    H264NalUnitDataLengthInvalid { len: usize },
    H264NalUnitLengthTooSmall { len: usize },
    H264DepacketizationNalUnitTypeUnknown { nal_unit_type: u8 },
    H264DepacketizationNalUnitTypeUnsupported { nal_unit_type_name: String },
    H264AggregationUnitHeaderInvalid { len: usize },
    H264AggregationUnitDataTooSmall { have: usize, need: usize },
    H264FragmentationUnitHeaderInvalid { len: usize },
    H264FragmentedStateAlreadyStarted,
    H264FragmentedStateNeverStarted,
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
            Error::H264NalUnitDataLengthInvalid { len } => {
                write!(f, "nal unit data length invalid (overflow): {len}")
            }
            Error::H264NalUnitLengthTooSmall { len } => {
                write!(
                    f,
                    "nal unit data length too small (must be at least one byte): {len}"
                )
            }
            Error::H264DepacketizationNalUnitTypeUnknown { nal_unit_type } => {
                write!(
                    f,
                    "encountered unknown nal unit type when depacketizing: {nal_unit_type}"
                )
            }
            Error::H264DepacketizationNalUnitTypeUnsupported { nal_unit_type_name } => {
                write!(
                    f,
                    "unsupported nal unit type (packetization mode not supported): {nal_unit_type_name}"
                )
            }
            Error::H264AggregationUnitHeaderInvalid { len } => {
                write!(
                    f,
                    "aggregation unit header too small (need 2 bytes for nal size): {len}"
                )
            }
            Error::H264AggregationUnitDataTooSmall { have, need } => {
                write!(
                    f,
                    "aggregation unit payload too small: {have} (need {need})"
                )
            }
            Error::H264FragmentationUnitHeaderInvalid { len } => {
                write!(
                    f,
                    "fragmentation unit header too small (need 1 byte): {len}"
                )
            }
            Error::H264FragmentedStateAlreadyStarted => {
                write!(
                    f,
                    "received fragmented unit with start bit set \
                        but never finished previous fragmented unit"
                )
            }
            Error::H264FragmentedStateNeverStarted => {
                write!(f, "received unexpected fragmented unit")
            }
        }
    }
}

impl std::error::Error for Error {}
