use std::collections::BTreeMap;

use crate::error::Error;
use crate::parse::Parse;
use crate::serialize::Serialize;

pub use bytes::Bytes;
pub use http::uri::Uri;

pub trait Message: Serialize + std::fmt::Display {
    type Metadata: Parse;

    fn new(metadata: Self::Metadata, headers: Headers, body: Option<Bytes>) -> Self;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Headers {
    map: BTreeMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn with_cseq(cseq: usize) -> Headers {
        Self::from_iter([("CSeq".to_string(), cseq.to_string())])
    }

    pub fn with_cseq_and_session(cseq: usize, session_id: &str) -> Headers {
        Self::from_iter([
            ("CSeq".to_string(), cseq.to_string()),
            ("Session".to_string(), session_id.to_string()),
        ])
    }

    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.map.insert(key, value)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn into_map(self) -> BTreeMap<String, String> {
        self.map
    }

    pub fn as_map(&self) -> &BTreeMap<String, String> {
        &self.map
    }
}

impl From<BTreeMap<String, String>> for Headers {
    #[inline]
    fn from(map: BTreeMap<String, String>) -> Self {
        Self { map }
    }
}

impl std::iter::FromIterator<(String, String)> for Headers {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(headers: I) -> Self {
        Self {
            map: BTreeMap::from_iter(headers),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Describe,
    Announce,
    Setup,
    Play,
    Pause,
    Record,
    Options,
    Redirect,
    Teardown,
    GetParameter,
    SetParameter,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Method::Describe => write!(f, "DESCRIBE"),
            Method::Announce => write!(f, "ANNOUNCE"),
            Method::Setup => write!(f, "SETUP"),
            Method::Play => write!(f, "PLAY"),
            Method::Pause => write!(f, "PAUSE"),
            Method::Record => write!(f, "RECORD"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Redirect => write!(f, "REDIRECT"),
            Method::Teardown => write!(f, "TEARDOWN"),
            Method::GetParameter => write!(f, "GET_PARAMETER"),
            Method::SetParameter => write!(f, "SET_PARAMETER"),
        }
    }
}

impl std::str::FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DESCRIBE" => Ok(Method::Describe),
            "ANNOUNCE" => Ok(Method::Announce),
            "SETUP" => Ok(Method::Setup),
            "PLAY" => Ok(Method::Play),
            "PAUSE" => Ok(Method::Pause),
            "RECORD" => Ok(Method::Record),
            "OPTIONS" => Ok(Method::Options),
            "REDIRECT" => Ok(Method::Redirect),
            "TEARDOWN" => Ok(Method::Teardown),
            "GET_PARAMETER" => Ok(Method::GetParameter),
            "SET_PARAMETER" => Ok(Method::SetParameter),
            _ => Err(Error::MethodUnknown {
                method: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Version {
    #[default]
    V1,
    V2,
    Unknown,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Version::V1 => write!(f, "1.0"),
            Version::V2 => write!(f, "2.0"),
            Version::Unknown => write!(f, "?"),
        }
    }
}

pub type StatusCode = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCategory {
    Informational,
    Success,
    Redirection,
    ClientError,
    ServerError,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Continue,
    Ok,
    Created,
    LowonStorageSpace,
    MultipleChoices,
    MovedPermanently,
    MovedTemporarily,
    SeeOther,
    UseProxy,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Gone,
    LengthRequired,
    PreconditionFailed,
    RequestEntityTooLarge,
    RequestUriTooLong,
    UnsupportedMediaType,
    InvalidParameter,
    IllegalConferenceIdentifier,
    NotEnoughBandwidth,
    SessionNotFound,
    MethodNotValidInThisState,
    HeaderFieldNotValid,
    InvalidRange,
    ParameterIsReadOnly,
    AggregateOperationNotAllowed,
    OnlyAggregateOperationAllowed,
    UnsupportedTransport,
    DestinationUnreachable,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    RTSPVersionNotSupported,
    OptionNotSupported,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", status_to_code(*self), status_to_reason(*self))
    }
}

pub(crate) fn status_to_code(status: Status) -> StatusCode {
    match status {
        Status::Continue => 100,
        Status::Ok => 200,
        Status::Created => 201,
        Status::LowonStorageSpace => 250,
        Status::MultipleChoices => 300,
        Status::MovedPermanently => 301,
        Status::MovedTemporarily => 302,
        Status::SeeOther => 303,
        Status::UseProxy => 305,
        Status::BadRequest => 400,
        Status::Unauthorized => 401,
        Status::PaymentRequired => 402,
        Status::Forbidden => 403,
        Status::NotFound => 404,
        Status::MethodNotAllowed => 405,
        Status::NotAcceptable => 406,
        Status::ProxyAuthenticationRequired => 407,
        Status::RequestTimeout => 408,
        Status::Gone => 410,
        Status::LengthRequired => 411,
        Status::PreconditionFailed => 412,
        Status::RequestEntityTooLarge => 413,
        Status::RequestUriTooLong => 414,
        Status::UnsupportedMediaType => 415,
        Status::InvalidParameter => 451,
        Status::IllegalConferenceIdentifier => 452,
        Status::NotEnoughBandwidth => 453,
        Status::SessionNotFound => 454,
        Status::MethodNotValidInThisState => 455,
        Status::HeaderFieldNotValid => 456,
        Status::InvalidRange => 457,
        Status::ParameterIsReadOnly => 458,
        Status::AggregateOperationNotAllowed => 459,
        Status::OnlyAggregateOperationAllowed => 460,
        Status::UnsupportedTransport => 461,
        Status::DestinationUnreachable => 462,
        Status::InternalServerError => 500,
        Status::NotImplemented => 501,
        Status::BadGateway => 502,
        Status::ServiceUnavailable => 503,
        Status::GatewayTimeout => 504,
        Status::RTSPVersionNotSupported => 505,
        Status::OptionNotSupported => 551,
    }
}

pub(crate) fn status_from_code(code: StatusCode) -> Option<Status> {
    match code {
        100 => Some(Status::Continue),
        200 => Some(Status::Ok),
        201 => Some(Status::Created),
        250 => Some(Status::LowonStorageSpace),
        300 => Some(Status::MultipleChoices),
        301 => Some(Status::MovedPermanently),
        302 => Some(Status::MovedTemporarily),
        303 => Some(Status::SeeOther),
        305 => Some(Status::UseProxy),
        400 => Some(Status::BadRequest),
        401 => Some(Status::Unauthorized),
        402 => Some(Status::PaymentRequired),
        403 => Some(Status::Forbidden),
        404 => Some(Status::NotFound),
        405 => Some(Status::MethodNotAllowed),
        406 => Some(Status::NotAcceptable),
        407 => Some(Status::ProxyAuthenticationRequired),
        408 => Some(Status::RequestTimeout),
        410 => Some(Status::Gone),
        411 => Some(Status::LengthRequired),
        412 => Some(Status::PreconditionFailed),
        413 => Some(Status::RequestEntityTooLarge),
        414 => Some(Status::RequestUriTooLong),
        415 => Some(Status::UnsupportedMediaType),
        451 => Some(Status::InvalidParameter),
        452 => Some(Status::IllegalConferenceIdentifier),
        453 => Some(Status::NotEnoughBandwidth),
        454 => Some(Status::SessionNotFound),
        455 => Some(Status::MethodNotValidInThisState),
        456 => Some(Status::HeaderFieldNotValid),
        457 => Some(Status::InvalidRange),
        458 => Some(Status::ParameterIsReadOnly),
        459 => Some(Status::AggregateOperationNotAllowed),
        460 => Some(Status::OnlyAggregateOperationAllowed),
        461 => Some(Status::UnsupportedTransport),
        462 => Some(Status::DestinationUnreachable),
        500 => Some(Status::InternalServerError),
        501 => Some(Status::NotImplemented),
        502 => Some(Status::BadGateway),
        503 => Some(Status::ServiceUnavailable),
        504 => Some(Status::GatewayTimeout),
        505 => Some(Status::RTSPVersionNotSupported),
        551 => Some(Status::OptionNotSupported),
        _ => None,
    }
}

pub(crate) fn status_to_reason(status: Status) -> &'static str {
    match status {
        Status::Continue => "Continue",
        Status::Ok => "OK",
        Status::Created => "Created",
        Status::LowonStorageSpace => "Low on Storage Space",
        Status::MultipleChoices => "Multiple Choices",
        Status::MovedPermanently => "Moved Permanently",
        Status::MovedTemporarily => "Moved Temporarily",
        Status::SeeOther => "See Other",
        Status::UseProxy => "Use Proxy",
        Status::BadRequest => "Bad Request",
        Status::Unauthorized => "Unauthorized",
        Status::PaymentRequired => "Payment Required",
        Status::Forbidden => "Forbidden",
        Status::NotFound => "Not Found",
        Status::MethodNotAllowed => "Method Not Allowed",
        Status::NotAcceptable => "Not Acceptable",
        Status::ProxyAuthenticationRequired => "Proxy Authentication Required",
        Status::RequestTimeout => "Request Timeout",
        Status::Gone => "Gone",
        Status::LengthRequired => "Length Required",
        Status::PreconditionFailed => "Precondition Failed",
        Status::RequestEntityTooLarge => "Request Entity Too Large",
        Status::RequestUriTooLong => "Request-URI Too Long",
        Status::UnsupportedMediaType => "Unsupported Media Type",
        Status::InvalidParameter => "Invalid parameter",
        Status::IllegalConferenceIdentifier => "Illegal Conference Identifier",
        Status::NotEnoughBandwidth => "Not Enough Bandwidth",
        Status::SessionNotFound => "Session Not Found",
        Status::MethodNotValidInThisState => "Method Not Valid In This State",
        Status::HeaderFieldNotValid => "Header Field Not Valid",
        Status::InvalidRange => "Invalid Range",
        Status::ParameterIsReadOnly => "Parameter Is Read-Only",
        Status::AggregateOperationNotAllowed => "Aggregate Operation Not Allowed",
        Status::OnlyAggregateOperationAllowed => "Only Aggregate Operation Allowed",
        Status::UnsupportedTransport => "Unsupported Transport",
        Status::DestinationUnreachable => "Destination Unreachable",
        Status::InternalServerError => "Internal Server Error",
        Status::NotImplemented => "Not Implemented",
        Status::BadGateway => "Bad Gateway",
        Status::ServiceUnavailable => "Service Unavailable",
        Status::GatewayTimeout => "Gateway Timeout",
        Status::RTSPVersionNotSupported => "RTSP Version Not Supported",
        Status::OptionNotSupported => "Option Not Supported",
    }
}
