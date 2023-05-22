use crate::codec::{CodecInfo, MediaAttributes};
use crate::error::{Error, Result};
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
    /* u= */
    pub uri: Option<String>,
    /* e= */
    pub email: Option<String>,
    /* p= */
    pub phone: Option<String>,
    /* s= */
    pub session_name: String,
    /* i= */
    pub session_description: Option<String>,
    /* c= */
    pub connection_network_type: NetworkType,
    pub connection_address_type: AddressType,
    pub connection_address: String,
    /* t= */
    pub timing: Timing,
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
            uri: None,
            email: None,
            phone: None,
            connection_network_type: NetworkType::Internet,
            connection_address_type: ip_addr_type(&destination),
            connection_address: destination.to_string(),
            tags: Vec::new(),
            timing: time_range.into(),
            media: Vec::new(),
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        /*
                Session description
           v=  (protocol version)
           o=  (originator and session identifier)
           s=  (session name)
           i=* (session information)
           u=* (URI of description)
           e=* (email address)
           p=* (phone number)
           c=* (connection information -- not required if included in
                all media descriptions)
           b=* (zero or more bandwidth information lines)
           One or more time descriptions:
             ("t=", "r=" and "z=" lines; see below)
           k=* (obsolete)
           a=* (zero or more session attribute lines)
           Zero or more media descriptions

        Time description
           t=  (time the session is active)
           r=* (zero or more repeat times)
           z=* (optional time zone offset line)

        Media description, if present
           m=  (media name and transport address)
           i=* (media title)
           c=* (connection information -- optional if included at
                session level)
           b=* (zero or more bandwidth information lines)
           k=* (obsolete)
           a=* (zero or more media attribute lines)

              */
        // TODO: medias not required
        // TODO: v= required (and always comes first)
        // TODO: o= required (originator)
        // TODO: s= required (name)
        // TODO: i= optional (session information)
        // TODO: u= optional (URI)
        // TODO: e= optional (email)
        // TODO: p= optional (phone)
        // TODO: c= optional if not in all medias (connection)
        // TODO: b= zero or more bandwidth information lines
        // TODO: a= zero or more attributes
        // TODO: t= at least one required (times)
        // TODO: r= zero or more repeat times
        // TODO: z= optional time zone offset
        // TODO: m= (zero or more)
        // TODO:    i= optional
        // TODO:    c= connection information (optional if included at session level)
        // TODO:    b= zero or more bandwidth information lines
        // TODO:    a= zero or more attributes

        let mut version: Option<Version> = None;
        let mut origin: Option<(String, String, String, NetworkType, AddressType, String)> = None;
        let mut session_name: Option<String> = None;
        let mut session_description: Option<String> = None;
        let mut uri: Option<String> = None;
        let mut email: Option<String> = None;
        let mut phone: Option<String> = None;
        let mut connection: Option<(NetworkType, AddressType, String)> = None;
        let mut timing: Vec<Timing> = Vec::new();
        let mut tags: Vec<Tag> = Vec::new();
        let mut media: Vec<Media> = Vec::new();

        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.len() < 2 {
                return Err(Error::LinePrefixInvalid {
                    line: line.to_string(),
                });
            }

            match &line[0..2] {
                "v=" => {
                    version = Some(line[2..].parse()?);
                }
                "o=" => {
                    let mut parts = line[2..].split(' ');
                    let mut next_or_invalid = || {
                        parts.next().ok_or_else(|| Error::OriginLineInvalid {
                            line: line.to_string(),
                        })
                    };

                    origin = Some((
                        next_or_invalid()?.to_string(),
                        next_or_invalid()?.to_string(),
                        next_or_invalid()?.to_string(),
                        next_or_invalid()?.parse()?,
                        next_or_invalid()?.parse()?,
                        next_or_invalid()?.to_string(),
                    ))
                }
                "s=" => {
                    session_name = Some(line[2..].to_string());
                }
                "i=" => {
                    let session_description_parsed = line[2..].to_string();
                    if let Some(media_item_in_scope) = media.last_mut() {
                        todo!()
                    } else {
                        session_description = Some(session_description_parsed);
                    };
                }
                "u=" => {
                    uri = Some(line[2..].to_string());
                }
                "e=" => {
                    email = Some(line[2..].to_string());
                }
                "p=" => {
                    phone = Some(line[2..].to_string());
                }
                "c=" => {
                    let mut parts = line[2..].split(' ');
                    let mut next_or_invalid = || {
                        parts.next().ok_or_else(|| Error::ConnectionLineInvalid {
                            line: line.to_string(),
                        })
                    };

                    let connection_parsed = (
                        next_or_invalid()?.parse()?,
                        next_or_invalid()?.parse()?,
                        next_or_invalid()?.to_string(),
                    );

                    if let Some(media_item_in_scope) = media.last_mut() {
                        todo!()
                    } else {
                        connection = Some(connection_parsed);
                    }
                }
                "b=" => {
                    // TODO: parse
                    if let Some(media_item_in_scope) = media.last_mut() {
                        todo!()
                    } else {
                        todo!()
                    }
                }
                "a=" => {
                    let tag = line[2..].parse()?;
                    if let Some(media_item_in_scope) = media.last_mut() {
                        media_item_in_scope.tags.push(tag);
                    } else {
                        tags.push(tag);
                    }
                }
                "t=" => {
                    timing.push(line[2..].parse()?);
                }
                "r=" => {
                    todo!()
                }
                "z=" => {
                    todo!()
                }
                "m=" => {
                    let mut parts = line[2..].split(' ');
                    let mut next_or_invalid = || {
                        parts.next().ok_or_else(|| Error::MediaLineInvalid {
                            line: line.to_string(),
                        })
                    };

                    let media_item = Media {
                        kind: next_or_invalid()?.parse()?,
                        port: next_or_invalid()?
                            .parse()
                            .map_err(|_| Error::MediaPortInvalid {
                                line: line.to_string(),
                            })?,
                        protocol: next_or_invalid()?.parse()?,
                        format: next_or_invalid()?.parse().map_err(|_| {
                            Error::MediaFormatInvalid {
                                line: line.to_string(),
                            }
                        })?,
                        tags: Vec::new(),
                    };

                    media.push(media_item);
                }
                _ => {
                    return Err(Error::LinePrefixInvalid {
                        line: line.to_string(),
                    })
                }
            }
        }

        // TODO: check if all required items are there
        // TODO: check if required items (if not in media) are there (bit complicated)

        todo!()
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
        // TODO: do not forget to serialize newly added items here!

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

        writeln!(f, "t={} {}", self.timing.start, self.timing.stop)?;

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
        // TODO: do not forget to serialize newly added items here!

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

impl std::str::FromStr for Timing {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.split_once(' ')
            .ok_or_else(|| Error::TimeMalformed {
                time: s.to_string(),
            })
            .and_then(|(start, stop)| {
                Ok(Timing {
                    start: start
                        .parse::<u64>()
                        .map_err(|_| Error::TimeDescriptionInvalid {
                            time: start.to_string(),
                        })?,
                    stop: stop
                        .parse::<u64>()
                        .map_err(|_| Error::TimeDescriptionInvalid {
                            time: stop.to_string(),
                        })?,
                })
            })
    }
}

impl From<TimeRange> for Timing {
    fn from(time_range: TimeRange) -> Timing {
        match time_range {
            TimeRange::Live => Timing { start: 0, stop: 0 },
            TimeRange::Playback { start, end } => Timing { start, stop: end },
        }
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

impl std::str::FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "0" => Ok(Version::V0),
            _ => Err(Error::VersionUnknown {
                version: s.to_string(),
            }),
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

impl std::str::FromStr for NetworkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "IP4" => Ok(NetworkType::Internet),
            _ => Err(Error::NetworkTypeUnknown {
                network_type: s.to_string(),
            }),
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

impl std::str::FromStr for AddressType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "IP4" => Ok(AddressType::IpV4),
            "IP6" => Ok(AddressType::IpV6),
            _ => Err(Error::AddressTypeUnknown {
                address_type: s.to_string(),
            }),
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

impl std::str::FromStr for Tag {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some((variable, value)) = s.split_once(':') {
            Ok(Tag::Value(variable.to_string(), value.to_string()))
        } else {
            Ok(Tag::Property(s.to_string()))
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

impl std::str::FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "recvonly" => Ok(Direction::ReceiveOnly),
            "sendonly" => Ok(Direction::SendOnly),
            "sendrecv" => Ok(Direction::SendAndReceive),
            _ => Err(Error::DirectionUnknown {
                direction: s.to_string(),
            }),
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

impl std::str::FromStr for Kind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "video" => Ok(Kind::Video),
            "audio" => Ok(Kind::Audio),
            "text" => Ok(Kind::Text),
            "application" => Ok(Kind::Application),
            "message" => Ok(Kind::Message),
            _ => Err(Error::KindUnknown {
                kind: s.to_string(),
            }),
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

impl std::str::FromStr for Protocol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "RTP/AVP" => Ok(Protocol::RtpAvp),
            "RTP/SAVP" => Ok(Protocol::RtpSAvp),
            _ => Err(Error::ProtocolUnknown {
                protocol: s.to_string(),
            }),
        }
    }
}
