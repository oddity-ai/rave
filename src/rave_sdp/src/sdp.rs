use crate::codec::{CodecInfo, MediaAttributes};
use crate::fmt::FMT_RTP_PAYLOAD_DYNAMIC;
use crate::ip::ip_addr_type;
use crate::time::unix_epoch_timestamp;
use crate::timing::TimeRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sdp {
    /* v= */
    pub version: Version,
    /* o= */
    pub origin_username: String,
    pub origin_session_id: String,
    pub origin_session_version: String,
    pub origin_network_type: NetworkType,
    pub origin_address_type: AddressType,
    pub origin_unicast_address: String,
    /* s= */
    pub session_name: String,
    /* i= */
    pub session_description: Option<String>,
    /* c= */
    pub connection_network_type: NetworkType,
    pub connection_address_type: AddressType,
    pub connection_address: String,
    /* t= */
    pub timing: (u64, u64),
    /* a= */
    pub tags: Vec<Tag>,
    /* ... */
    pub media: Vec<Media>,
}

impl Sdp {
    pub fn new(
        origin: std::net::IpAddr,
        name: String,
        destination: std::net::IpAddr,
        time_range: TimeRange,
    ) -> Self {
        Self {
            version: Version::V0,
            origin_username: "-".to_string(),
            origin_session_id: unix_epoch_timestamp().to_string(),
            origin_session_version: 0_u64.to_string(),
            origin_network_type: NetworkType::Internet,
            origin_address_type: ip_addr_type(&origin),
            origin_unicast_address: origin.to_string(),
            session_name: name,
            session_description: None,
            connection_network_type: NetworkType::Internet,
            connection_address_type: ip_addr_type(&destination),
            connection_address: destination.to_string(),
            tags: Vec::new(),
            timing: time_range.into(),
            media: Vec::new(),
        }
    }

    pub fn with_username(mut self, username: &str) -> Self {
        self.origin_username = username.to_string();
        self
    }

    pub fn with_session_version(mut self, version: usize) -> Self {
        self.origin_session_version = version.to_string();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.session_description = Some(description.to_string());
        self
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = Tag>) -> Self {
        self.tags.extend(tags);
        self
    }

    pub fn with_media(
        mut self,
        kind: Kind,
        port: u16,
        protocol: Protocol,
        codec_info: CodecInfo,
        direction: Direction,
    ) -> Self {
        let mut tags = codec_info.media_attributes();
        tags.push(Tag::Property(direction.to_string()));

        self.media.push(Media {
            kind,
            port,
            protocol,
            format: FMT_RTP_PAYLOAD_DYNAMIC,
            tags,
        });
        self
    }
}

impl std::fmt::Display for Sdp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "v={}", self.version)?;
        writeln!(
            f,
            "o={} {} {} {} {} {}",
            self.origin_username,
            self.origin_session_id,
            self.origin_session_version,
            self.origin_network_type,
            self.origin_address_type,
            self.origin_unicast_address
        )?;

        writeln!(f, "s={}", self.session_name)?;
        if let Some(session_description) = self.session_description.as_ref() {
            writeln!(f, "i={session_description}")?;
        }

        writeln!(
            f,
            "c={} {} {}",
            self.connection_network_type, self.connection_address_type, self.connection_address
        )?;

        writeln!(f, "t={} {}", self.timing.0, self.timing.1)?;

        for tag in &self.tags {
            writeln!(f, "a={tag}")?;
        }

        for media in &self.media {
            write!(f, "{media}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Media {
    /* m= */
    pub kind: Kind,
    pub port: u16,
    pub protocol: Protocol,
    pub format: usize,
    /* a= */
    pub tags: Vec<Tag>,
}

impl std::fmt::Display for Media {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "m={} {} {} {}",
            self.kind, self.port, self.protocol, self.format,
        )?;

        for tag in &self.tags {
            writeln!(f, "a={tag}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timing {
    pub start: u64,
    pub stop: u64,
}

impl std::fmt::Display for Timing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.start, self.stop)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    V0,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Version::V0 => write!(f, "0"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Internet,
}

impl std::fmt::Display for NetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NetworkType::Internet => write!(f, "IN"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressType {
    IpV4,
    IpV6,
}

impl std::fmt::Display for AddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AddressType::IpV4 => write!(f, "IP4"),
            AddressType::IpV6 => write!(f, "IP6"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Property(String),
    Value(String, String),
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Tag::Property(value) => write!(f, "{value}"),
            Tag::Value(variable, value) => write!(f, "{variable}:{value}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ReceiveOnly,
    SendOnly,
    SendAndReceive,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Direction::ReceiveOnly => write!(f, "recvonly"),
            Direction::SendOnly => write!(f, "sendonly"),
            Direction::SendAndReceive => write!(f, "sendrecv"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Video,
    Audio,
    Text,
    Application,
    Message,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Kind::Video => write!(f, "video"),
            Kind::Audio => write!(f, "audio"),
            Kind::Text => write!(f, "text"),
            Kind::Application => write!(f, "application"),
            Kind::Message => write!(f, "message"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    RtpAvp,
    RtpSAvp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Protocol::RtpAvp => write!(f, "RTP/AVP"),
            Protocol::RtpSAvp => write!(f, "RTP/SAVP"),
        }
    }
}
