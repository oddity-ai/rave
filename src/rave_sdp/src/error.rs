pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // TODO
    LinePrefixInvalid { line: String },
    OriginLineInvalid { line: String },
    ConnectionLineInvalid { line: String },
    MediaLineInvalid { line: String },
    MediaPortInvalid { line: String },
    MediaFormatInvalid { line: String },
    TimeMalformed { time: String },
    TimeDescriptionInvalid { time: String },
    VersionUnknown { version: String },
    NetworkTypeUnknown { network_type: String },
    AddressTypeUnknown { address_type: String },
    DirectionUnknown { direction: String },
    KindUnknown { kind: String },
    ProtocolUnknown { protocol: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::LinePrefixInvalid { line } => {
                write!(f, "line does not start with a valid prefix: {line}")
            }
            Error::OriginLineInvalid { line } => {
                write!(f, "origin line is invalid: {line}")
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
            Error::MediaFormatInvalid { line } => {
                write!(
                    f,
                    "media item has format identifier that is invalid (not an integer): {line}"
                )
            }
            Error::ConnectionLineInvalid { line } => {
                write!(f, "connection line is invalid: {line}")
            }
            Error::TimeMalformed { time } => write!(f, "time field malformed: {time}"),
            Error::TimeDescriptionInvalid { time } => {
                write!(f, "time description not a valid integer: {time}")
            }
            Error::VersionUnknown { version } => write!(f, "version unknown: {version}"),
            Error::NetworkTypeUnknown { network_type } => {
                write!(f, "network type unknown: {network_type}")
            }
            Error::AddressTypeUnknown { address_type } => {
                write!(f, "address type unknown: {address_type}")
            }
            Error::DirectionUnknown { direction } => write!(f, "direction unknown: {direction}"),
            Error::KindUnknown { kind } => write!(f, "media kind unknown: {kind}"),
            Error::ProtocolUnknown { protocol } => write!(f, "protocol unknown: {protocol}"),
        }
    }
}

impl std::error::Error for Error {}
