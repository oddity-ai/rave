use crate::message::Uri;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// An error occurred decoding the header due to incorrect usage of text encoding by the sender.
    Encoding,
    /// The request line of the head part is malformed.
    RequestLineMalformed { line: String },
    /// The header first line does have a method and target URI, but it does not have a version,
    /// which is the required third part of the first line of the head.
    VersionMissing { line: String },
    /// The response status line does have a version, but does not have a status code which is
    /// required.
    StatusCodeMissing { line: String },
    /// The specified method is not a valid method.
    MethodUnknown { method: String },
    /// The header first line does have a method, but it does not have a target URI, which is the
    /// required second part of the first line of the head.
    UriMissing { line: String },
    /// The header first line has a Request-URI, but it could not be parsed correctly.
    UriMalformed { line: String, uri: String },
    /// The Request-URI is correct, but represents a relative path, which is not allowed in RTSP.
    UriNotAbsolute { uri: Uri },
    /// The response status line has a version and status code, but is missing a reason phrase which
    /// is required.
    ReasonPhraseMissing { line: String },
    /// The version specifier is incorrect. It should start with "RTSP/" followed by a digit, "."
    /// and another digit.
    VersionMalformed { line: String, version: String },
    /// The provided status code is not an unsigned integer or cannot be converted to one. It must
    /// be a 3-digit non-negative number.
    StatusCodeNotInteger { line: String, status_code: String },
    /// Header line is malformed.
    HeaderMalformed { line: String },
    /// The Content-Length header is missing, but it is required.
    ContentLengthMissing,
    /// The Content-Length header is not an integer value, or cannot be converted to an unsigned
    /// integer.
    ContentLengthNotInteger { value: String },
    /// This occurs when the caller invokes the state machine with a state that signals that parsing
    /// the head part of the request was already done before.
    HeadAlreadyDone,
    /// This occurs when the caller invokes the state machine with a state that signals that parsing
    /// the body part of the request was already done before.
    BodyAlreadyDone,
    /// Metadata was not parsed for some reason.
    MetadataNotParsed,
    /// This occurs when the caller tries to turn the parser into an actual request, but the parser
    /// was not ready yet.
    NotDone,
    /// This occurs when trying to serialize a request that does not have a known version.
    VersionUnknown,
    /// Transport header does not have protocol and profile string. The transport must start with
    /// `RTP/AVP`, where `RTP` denotes the protocol and `AVP` the profile.
    TransportProtocolProfileMissing { value: String },
    /// Transport header contains unknown lower protocol. Use either `TCP` or `UDP`.
    TransportLowerUnknown { value: String },
    /// Transport header contains unknown parameter. Please see RFC 2326 Section 12.39 for a list of
    /// permissable parameters.
    TransportParameterUnknown { var: String },
    /// Transport header contains parameter that should have a value, but does not have one.
    TransportParameterValueMissing { var: String },
    /// Transport header contains parameter with invalid value.
    TransportParameterValueInvalid { var: String, val: String },
    /// Transport header contains invalid or malformed parameter.
    TransportParameterInvalid { parameter: String },
    /// Transport header channel is malformed.
    TransportChannelMalformed { value: String },
    /// Transport header port is malformed.
    TransportPortMalformed { value: String },
    /// Tried to parse interleaved data but there is no interleaved header. Interleaved packets
    /// always start with `$` (0x24).
    InterleavedInvalid,
    /// Interleaved payload too large. The size cannot be larger than the maximum value of a 16-bit
    /// unsigned integer.
    InterleavedPayloadTooLarge,
    /// Range header value malformed.
    RangeMalformed { value: String },
    /// Parser does not support provided `Range` header unit.
    RangeUnitNotSupported { value: String },
    /// Parser does not support effective time in `Range` header.
    RangeTimeNotSupported { value: String },
    /// The NPT time (either the from or to part of the time specifier)
    /// is malformed.
    RangeNptTimeMalfored { value: String },
    /// RTP Info must always contain a URL.
    RtpInfoUrlMissing { value: String },
    /// RTP Info parameter is not known. This means that the RTP part contains an unknown or
    /// non-existant parameter variable.
    RtpInfoParameterUnknown { value: String },
    /// RTP Info parameter is invalid. This happens, for example, when the `seq` parameter is not an
    /// integer.
    RtpInfoParameterInvalid { value: String },
    /// RTP Info contains unexpected extra parameter.
    RtpInfoParameterUnexpected { value: String },
    /// Underlying socket was shut down. This is not really an error and consumers are expected to
    /// handle it gracefully.
    Shutdown,
    /// I/O error occurred.
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Encoding => write!(f, "encoding incorrect"),
            Error::RequestLineMalformed { line } => write!(f, "request line malformed: {}", &line),
            Error::VersionMissing { line } => {
                write!(f, "version missing in request line: {}", &line)
            }
            Error::StatusCodeMissing { line } => {
                write!(f, "status code missing in response line: {}", &line)
            }
            Error::MethodUnknown { method } => write!(f, "method unknown: {}", &method),
            Error::UriMissing { line } => write!(f, "uri missing in request line: {}", &line),
            Error::UriMalformed { line, uri } => {
                write!(f, "uri malformed: {} (in line: {})", &uri, &line)
            }
            Error::UriNotAbsolute { uri } => {
                write!(f, "uri must be absolute, but it is relative: {}", &uri)
            }
            Error::ReasonPhraseMissing { line } => {
                write!(f, "reason phrase missing in response line: {}", &line)
            }
            Error::VersionMalformed { line, version } => {
                write!(f, "version malformed: {} (in line: {})", &version, &line)
            }
            Error::StatusCodeNotInteger { line, status_code } => write!(
                f,
                "response has invalid status code: {} (in response line: {})",
                &status_code, &line
            ),
            Error::HeaderMalformed { line } => write!(f, "header line malformed: {}", &line),
            Error::ContentLengthMissing => write!(f, "request does not have Content-Length header"),
            Error::ContentLengthNotInteger { value } => write!(
                f,
                "request has invalid value for Content-Length: {}",
                &value
            ),
            Error::HeadAlreadyDone => write!(f, "head already done (cycle in state machine)"),
            Error::BodyAlreadyDone => write!(f, "body already done (cycle in state machine)"),
            Error::MetadataNotParsed => write!(f, "metadata not parsed"),
            Error::NotDone => write!(f, "parser not done yet"),
            Error::VersionUnknown => write!(f, "response has unknown version"),
            Error::TransportProtocolProfileMissing { value } => {
                write!(f, "transport protocol and/or profile missing: {}", &value)
            }
            Error::TransportLowerUnknown { value } => {
                write!(f, "transport lower protocol unknown: {}", &value)
            }
            Error::TransportParameterUnknown { var } => {
                write!(f, "transport parameter unknown: {}", &var)
            }
            Error::TransportParameterValueMissing { var } => write!(
                f,
                "transport parameter should have value but does not (var: {})",
                &var
            ),
            Error::TransportParameterValueInvalid { var, val } => write!(
                f,
                "transport parameter value is invalid or malformed (var: {}, val: {})",
                &var, &val
            ),
            Error::TransportParameterInvalid { parameter } => {
                write!(f, "transport parameter invalid: {}", &parameter)
            }
            Error::TransportChannelMalformed { value } => {
                write!(f, "transport channel malformed: {}", &value)
            }
            Error::TransportPortMalformed { value } => {
                write!(f, "transport port malformed: {}", &value)
            }
            Error::InterleavedInvalid => write!(
                f,
                "interleaved data does not have valid header magic character"
            ),
            Error::InterleavedPayloadTooLarge => write!(f, "interleaved payload too large"),
            Error::RangeMalformed { value } => write!(f, "range malformed: {value}"),
            Error::RangeUnitNotSupported { value } => {
                write!(f, "range unit not supported: {}", &value)
            }
            Error::RangeTimeNotSupported { value } => {
                write!(f, "range time not supported: {}", &value)
            }
            Error::RangeNptTimeMalfored { value } => {
                write!(f, "range npt time malformed: {}", &value)
            }
            Error::RtpInfoUrlMissing { value } => write!(f, "rtp info url missing: {}", &value),
            Error::RtpInfoParameterUnknown { value } => {
                write!(f, "rtp info parameter unknown: {}", &value)
            }
            Error::RtpInfoParameterInvalid { value } => {
                write!(f, "rtp info parameter invalid: {}", &value)
            }
            Error::RtpInfoParameterUnexpected { value } => {
                write!(f, "rtp info contains unexpected parameter: {}", &value)
            }
            Error::Shutdown => write!(f, "underlying socket was shut down"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl std::error::Error for Error {}
