pub type Result<T> = std::result::Result<T, Error>;

/// Represent error when parsing or serializing SDP.
#[derive(Debug)]
pub enum Error {
    AddressTypeUnknown { address_type: String },
    BandwidthLineMalformed { line: String },
    BandwidthTypeUnknown { bandwidth_type: String },
    BandwidthValueInvalid { bandwidth: String },
    ConnectionAddressTtlInvalid { ttl: String },
    ConnectionAddressMulticastInvalid { multicast: String },
    ConnectionLineInvalid { line: String },
    ConnectionMissing,
    DirectionUnknown { direction: String },
    KindUnknown { kind: String },
    LinePrefixInvalid { line: String },
    MediaFormatInvalid { line: String },
    MediaLineInvalid { line: String },
    MediaPortInvalid { line: String },
    NetworkTypeUnknown { network_type: String },
    OriginLineInvalid { line: String },
    OriginMissing,
    OriginUnicastAddressInvalid { unicast_address: String },
    ProtocolUnknown { protocol: String },
    RepeatTimesLineMalformed { line: String },
    SessionNameMissing,
    TimeDescriptionInvalid { time: String },
    TimeInvalid { time: String },
    TimeMalformed { time: String },
    TimeZoneAdjustmentsLineMalformed { line: String },
    TimezoneAdjustmentsWithoutRepeatTimes,
    TimeZoneAdjustmentTimeInvalid { time: String },
    TimeActiveMissing,
    TooManyMediaItems,
    VersionMissing,
    VersionUnknown { version: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::AddressTypeUnknown { address_type } => {
                write!(f, "address type unknown: {address_type}")
            }
            Error::BandwidthLineMalformed { line } => write!(
                f,
                "bandwidth line malformed (must be in format <bwtype>:<bandwidth>): {line}"
            ),
            Error::BandwidthTypeUnknown { bandwidth_type } => {
                write!(f, "bandwidth type unknown: {bandwidth_type}")
            }
            Error::BandwidthValueInvalid { bandwidth } => {
                write!(f, "bandwidth value not a valid integer: {bandwidth}")
            }
            Error::ConnectionLineInvalid { line } => {
                write!(f, "connection line is invalid: {line}")
            }
            Error::ConnectionAddressTtlInvalid { ttl } => {
                write!(f, "connection address ttl invalid: {ttl}")
            }
            Error::ConnectionAddressMulticastInvalid { multicast } => {
                write!(
                    f,
                    "connection address multicast number invalid: {multicast}"
                )
            }
            Error::ConnectionMissing => write!(
                f,
                "connection missing in global info or one or more media items"
            ),
            Error::DirectionUnknown { direction } => write!(f, "direction unknown: {direction}"),
            Error::KindUnknown { kind } => write!(f, "media kind unknown: {kind}"),
            Error::LinePrefixInvalid { line } => {
                write!(f, "line does not start with a valid prefix: {line}")
            }
            Error::MediaFormatInvalid { line } => {
                write!(
                    f,
                    "media item has format identifier that is invalid (not an integer): {line}"
                )
            }
            Error::MediaLineInvalid { line } => {
                write!(f, "media line is invalid: {line}")
            }
            Error::MediaPortInvalid { line } => {
                write!(
                    f,
                    "media item has port that is invalid (not an integer): {line}"
                )
            }
            Error::NetworkTypeUnknown { network_type } => {
                write!(f, "network type unknown: {network_type}")
            }
            Error::OriginLineInvalid { line } => {
                write!(f, "origin line is invalid: {line}")
            }
            Error::OriginMissing => write!(f, "origin missing"),
            Error::OriginUnicastAddressInvalid { unicast_address } => {
                write!(
                    f,
                    "origin specifies invalid unicast address: {unicast_address}"
                )
            }
            Error::ProtocolUnknown { protocol } => write!(f, "protocol unknown: {protocol}"),
            Error::RepeatTimesLineMalformed { line } => {
                write!(f, "repeat times line malformed: {line}")
            }
            Error::SessionNameMissing => write!(f, "session name missing"),
            Error::TimeDescriptionInvalid { time } => {
                write!(f, "time description not a valid integer: {time}")
            }
            Error::TimeInvalid { time } => {
                write!(f, "time not a valid integer: {time}")
            }
            Error::TimeMalformed { time } => write!(f, "time field malformed: {time}"),
            Error::TimeZoneAdjustmentsLineMalformed { line } => {
                write!(f, "timezone adjustment line malformed: {line}")
            }
            Error::TimezoneAdjustmentsWithoutRepeatTimes => write!(
                f,
                "encountered timezone adjustments without repeat times (z= must follow r=)"
            ),
            Error::TimeZoneAdjustmentTimeInvalid { time } => {
                write!(f, "timezone adjustment time not a valid integer: {time}")
            }
            Error::TimeActiveMissing => write!(f, "timing missing"),
            Error::TooManyMediaItems => write!(
                f,
                "too many media items (ran out of dynamic payload assignments)"
            ),
            Error::VersionMissing => write!(f, "version missing"),
            Error::VersionUnknown { version } => write!(f, "version unknown: {version}"),
        }
    }
}

impl std::error::Error for Error {}
