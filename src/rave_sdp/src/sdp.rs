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
    pub origin: Origin,
    /* s= */
    pub session_name: String,
    /* i= */
    pub session_description: Option<String>,
    /* u= */
    pub uri: Option<String>,
    /* e= */
    pub email: Option<String>,
    /* p= */
    pub phone: Option<String>,
    /* c= */
    pub connection: Option<Connection>,
    /* b= */
    pub bandwidth: Vec<Bandwidth>,
    /* t= */
    pub timing: Vec<Timing>,
    /* r= */
    pub repeat_times: Vec<RepeatTimes>,
    /* z= */
    pub timezone_adjustments: Option<TimeZoneAdjustments>,
    /* a= */
    pub tags: Vec<Tag>,
    /* m= */
    pub media: Vec<MediaItem>,
}

impl Sdp {
    pub fn new(
        origin: std::net::IpAddr,
        name: &str,
        destination: std::net::IpAddr,
        time_range: TimeRange,
    ) -> Self {
        Self {
            version: Default::default(),
            origin: Origin::from(origin),
            session_name: name.to_string(),
            session_description: None,
            uri: None,
            email: None,
            phone: None,
            connection: Some(Connection::from(destination)),
            bandwidth: Vec::new(),
            timing: vec![Timing::from(time_range)],
            repeat_times: Vec::new(),
            timezone_adjustments: None,
            tags: Vec::new(),
            media: Vec::new(),
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        let mut version: Option<Version> = None;
        let mut origin: Option<Origin> = None;
        let mut session_name: Option<String> = None;
        let mut session_description: Option<String> = None;
        let mut uri: Option<String> = None;
        let mut email: Option<String> = None;
        let mut phone: Option<String> = None;
        let mut connection: Option<Connection> = None;
        let mut bandwidth: Vec<Bandwidth> = Vec::new();
        let mut timing: Vec<Timing> = Vec::new();
        let mut repeat_times: Vec<RepeatTimes> = Vec::new();
        let mut timezone_adjustments: Option<TimeZoneAdjustments> = None;
        let mut tags: Vec<Tag> = Vec::new();
        let mut media: Vec<MediaItem> = Vec::new();

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

            // TODO: order
            match &line[0..2] {
                "v=" => {
                    version = Some(line[2..].parse()?);
                }
                "o=" => {
                    origin = Some(line[2..].parse()?);
                }
                "s=" => {
                    session_name = Some(line[2..].to_string());
                }
                "i=" => {
                    let parsed = line[2..].to_string();
                    if let Some(media_item_in_scope) = media.last_mut() {
                        media_item_in_scope.title = Some(parsed);
                    } else {
                        session_description = Some(parsed);
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
                    let parsed = line[2..].parse()?;
                    if let Some(media_item_in_scope) = media.last_mut() {
                        media_item_in_scope.connection = Some(parsed);
                    } else {
                        connection = Some(parsed);
                    }
                }
                "b=" => {
                    let parsed = line[2..].parse()?;
                    if let Some(media_item_in_scope) = media.last_mut() {
                        media_item_in_scope.bandwidth.push(parsed);
                    } else {
                        bandwidth.push(parsed);
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
                    repeat_times.push(line[2..].parse()?);
                }
                "z=" => {
                    if repeat_times.is_empty() {
                        return Err(Error::TimezoneAdjustmentsWithoutRepeatTimes);
                    }
                    timezone_adjustments = Some(line[2..].parse()?);
                }
                "m=" => {
                    media.push(MediaItem {
                        media: line[2..].parse()?,
                        title: None,
                        connection: None,
                        bandwidth: Vec::new(),
                        tags: Vec::new(),
                    });
                }
                _ => {
                    return Err(Error::LinePrefixInvalid {
                        line: line.to_string(),
                    })
                }
            }
        }

        let version = version.ok_or(Error::VersionMissing)?;
        let origin = origin.ok_or(Error::OriginMissing)?;
        let session_name = session_name.ok_or(Error::SessionNameMissing)?;
        if timing.is_empty() {
            return Err(Error::TimingMissing);
        }
        if connection.is_none() && !media.iter().all(|media| media.connection.is_some()) {
            return Err(Error::ConnectionMissing);
        }

        Ok(Sdp {
            version,
            origin,
            session_name,
            session_description,
            uri,
            email,
            phone,
            connection,
            bandwidth,
            timing,
            repeat_times,
            timezone_adjustments,
            tags,
            media,
        })
    }

    pub fn with_username(mut self, username: &str) -> Self {
        self.origin.username = username.to_string();
        self
    }

    pub fn with_session_version(mut self, version: usize) -> Self {
        self.origin.session_version = version.to_string();
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
        title: &str,
        port: u16,
        protocol: Protocol,
        codec_info: CodecInfo,
        direction: Direction,
    ) -> Self {
        let mut tags = codec_info.media_attributes();
        tags.push(Tag::Property(direction.to_string()));

        self.media.push(MediaItem {
            media: Media {
                kind,
                port,
                protocol,
                format: FMT_RTP_PAYLOAD_DYNAMIC,
            },
            title: Some(title.to_string()),
            connection: None,
            bandwidth: Vec::new(),
            tags,
        });
        self
    }
}

impl std::fmt::Display for Sdp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "v={}", self.version)?;
        writeln!(f, "o={}", self.origin)?;
        writeln!(f, "s={}", self.session_name)?;
        if let Some(session_description) = self.session_description.as_ref() {
            writeln!(f, "i={session_description}")?;
        }
        if let Some(uri) = self.uri.as_ref() {
            writeln!(f, "u={uri}")?;
        }
        if let Some(email) = self.email.as_ref() {
            writeln!(f, "e={email}")?;
        }
        if let Some(phone) = self.phone.as_ref() {
            writeln!(f, "p={phone}")?;
        }
        if let Some(connection) = self.connection.as_ref() {
            writeln!(f, "c={connection}")?;
        }
        for bandwidth in &self.bandwidth {
            writeln!(f, "b={bandwidth}")?;
        }
        for timing in &self.timing {
            writeln!(f, "t={timing}")?;
        }
        for repeat_times_item in &self.repeat_times {
            writeln!(f, "r={repeat_times_item}")?;
        }
        if let Some(timezone_adjustments) = self.timezone_adjustments.as_ref() {
            writeln!(f, "z={timezone_adjustments}")?;
        }
        for tag in &self.tags {
            writeln!(f, "a={tag}")?;
        }
        for media in &self.media {
            write!(f, "{media}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Version {
    #[default]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub username: String,
    pub session_id: String,
    pub session_version: String,
    pub network_type: NetworkType,
    pub address_type: AddressType,
    pub unicast_address: String,
}

impl From<std::net::IpAddr> for Origin {
    fn from(ip_addr: std::net::IpAddr) -> Self {
        Self {
            username: "-".to_string(),
            session_id: unix_epoch_timestamp().to_string(),
            session_version: 0_u64.to_string(),
            network_type: NetworkType::Internet,
            address_type: ip_addr_type(&ip_addr),
            unicast_address: ip_addr.to_string(),
        }
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} {} {} {} {}",
            self.username,
            self.session_id,
            self.session_version,
            self.network_type,
            self.address_type,
            self.unicast_address
        )
    }
}

impl std::str::FromStr for Origin {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[inline(always)]
        fn next_or_invalid<'it>(
            line: &str,
            it: &mut impl Iterator<Item = &'it str>,
        ) -> Result<&'it str> {
            it.next().ok_or_else(|| Error::OriginLineInvalid {
                line: line.to_string(),
            })
        }

        let mut parts = s.split(' ');
        Ok(Origin {
            username: next_or_invalid(s, &mut parts)?.to_string(),
            session_id: next_or_invalid(s, &mut parts)?.to_string(),
            session_version: next_or_invalid(s, &mut parts)?.to_string(),
            network_type: next_or_invalid(s, &mut parts)?.parse()?,
            address_type: next_or_invalid(s, &mut parts)?.parse()?,
            unicast_address: next_or_invalid(s, &mut parts)?.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub network_type: NetworkType,
    pub address_type: AddressType,
    pub address: String,
}

impl From<std::net::IpAddr> for Connection {
    fn from(ip_addr: std::net::IpAddr) -> Self {
        Connection {
            network_type: NetworkType::Internet,
            address_type: ip_addr_type(&ip_addr),
            address: ip_addr.to_string(),
        }
    }
}

impl std::fmt::Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} {}",
            self.network_type, self.address_type, self.address,
        )
    }
}

impl std::str::FromStr for Connection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[inline(always)]
        fn next_or_invalid<'it>(
            line: &str,
            it: &mut impl Iterator<Item = &'it str>,
        ) -> Result<&'it str> {
            it.next().ok_or_else(|| Error::ConnectionLineInvalid {
                line: line.to_string(),
            })
        }

        let mut parts = s.split(' ');
        Ok(Connection {
            network_type: next_or_invalid(s, &mut parts)?.parse()?,
            address_type: next_or_invalid(s, &mut parts)?.parse()?,
            address: next_or_invalid(s, &mut parts)?.to_string(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bandwidth {
    ConferenceTotal(usize),
    ApplicationSpecific(usize),
}

impl std::fmt::Display for Bandwidth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Bandwidth::ConferenceTotal(kilobitrate) => write!(f, "CT:{kilobitrate}"),
            Bandwidth::ApplicationSpecific(kilobitrate) => write!(f, "AS:{kilobitrate}"),
        }
    }
}

impl std::str::FromStr for Bandwidth {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[inline(always)]
        fn parse_bandwidth(bs: &str) -> Result<usize> {
            bs.parse::<usize>()
                .map_err(|_| Error::BandwidthValueInvalid {
                    bandwidth: bs.to_string(),
                })
        }

        let (bandwidth_type, bandwidth) =
            s.split_once(':')
                .ok_or_else(|| Error::BandwidthLineMalformed {
                    line: s.to_string(),
                })?;

        match bandwidth_type {
            "CT" => Ok(Bandwidth::ConferenceTotal(parse_bandwidth(bandwidth)?)),
            "AS" => Ok(Bandwidth::ApplicationSpecific(parse_bandwidth(bandwidth)?)),
            _ => Err(Error::BandwidthTypeUnknown {
                bandwidth_type: bandwidth_type.to_string(),
            }),
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatTimes {
    pub repeat_interval: u64,
    pub active_duration: u64,
    pub offsets: Vec<u64>,
}

impl std::fmt::Display for RepeatTimes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let offsets_formatted = if !self.offsets.is_empty() {
            "0".to_string()
        } else {
            self.offsets
                .iter()
                .map(|offset| format!("{offset}"))
                .collect::<Vec<_>>()
                .join(" ")
        };
        write!(
            f,
            "{} {} {}",
            self.repeat_interval, self.active_duration, offsets_formatted
        )
    }
}

impl std::str::FromStr for RepeatTimes {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[inline(always)]
        fn next_time<'it>(line: &str, it: &mut impl Iterator<Item = &'it str>) -> Result<u64> {
            parse_time(it.next().ok_or_else(|| Error::RepeatTimesLineMalformed {
                line: line.to_string(),
            })?)
        }

        let mut parts = s.split(' ');
        let mut repeat_times = RepeatTimes {
            repeat_interval: next_time(s, &mut parts)?,
            active_duration: next_time(s, &mut parts)?,
            offsets: vec![next_time(s, &mut parts)?],
        };

        for offset in parts {
            repeat_times.offsets.push(parse_time(offset)?);
        }

        Ok(repeat_times)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZoneAdjustments(Vec<TimeZoneAdjustment>);

impl std::fmt::Display for TimeZoneAdjustments {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let TimeZoneAdjustments(timezone_adjustments) = self;
        if !timezone_adjustments.is_empty() {
            write!(
                f,
                "{}",
                timezone_adjustments
                    .iter()
                    .map(|timezone_adjustment| format!("{timezone_adjustment}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        } else {
            write!(f, "0 0")
        }
    }
}

impl std::str::FromStr for TimeZoneAdjustments {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split(' ');
        Ok(TimeZoneAdjustments(
            std::iter::from_fn(|| match (parts.next(), parts.next()) {
                (Some(time), Some(offset)) => Some(
                    TimeZoneAdjustment::from_time_and_offset_strings(time, offset),
                ),
                (None, None) => None,
                _ => Some(Err(Error::TimeZoneAdjustmentsLineMalformed {
                    line: s.to_string(),
                })),
            })
            .collect::<Result<Vec<_>>>()?,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeZoneAdjustment {
    time: u64,
    offset: i64,
}

impl TimeZoneAdjustment {
    pub fn from_time_and_offset_strings(time: &str, offset: &str) -> Result<Self> {
        Ok(TimeZoneAdjustment {
            time: time
                .parse()
                .map_err(|_| Error::TimeZoneAdjustmentTimeInvalid {
                    time: time.to_string(),
                })?,
            offset: parse_time(offset)?,
        })
    }
}

impl std::fmt::Display for TimeZoneAdjustment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.time, self.offset)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Media {
    pub kind: Kind,
    pub port: u16,
    pub protocol: Protocol,
    pub format: usize,
}

impl std::fmt::Display for Media {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} {} {}",
            self.kind, self.port, self.protocol, self.format,
        )
    }
}

impl std::str::FromStr for Media {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[inline(always)]
        fn next_or_invalid<'it>(
            line: &str,
            it: &mut impl Iterator<Item = &'it str>,
        ) -> Result<&'it str> {
            it.next().ok_or_else(|| Error::MediaLineInvalid {
                line: line.to_string(),
            })
        }

        let mut parts = s.split(' ');
        Ok(Media {
            kind: next_or_invalid(s, &mut parts)?.parse()?,
            port: next_or_invalid(s, &mut parts)?
                .parse()
                .map_err(|_| Error::MediaPortInvalid {
                    line: s.to_string(),
                })?,
            protocol: next_or_invalid(s, &mut parts)?.parse()?,
            format: next_or_invalid(s, &mut parts)?.parse().map_err(|_| {
                Error::MediaFormatInvalid {
                    line: s.to_string(),
                }
            })?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaItem {
    /* m= */
    pub media: Media,
    /* i= */
    pub title: Option<String>,
    /* c = */
    pub connection: Option<Connection>,
    /* b= */
    pub bandwidth: Vec<Bandwidth>,
    /* a= */
    pub tags: Vec<Tag>,
}

impl std::fmt::Display for MediaItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "m={}", self.media)?;
        if let Some(title) = self.title.as_ref() {
            writeln!(f, "i={title}")?;
        }
        if let Some(connection) = self.connection.as_ref() {
            writeln!(f, "c={connection}")?;
        }
        for bandwidth in &self.bandwidth {
            writeln!(f, "b={bandwidth}")?;
        }
        for tag in &self.tags {
            writeln!(f, "a={tag}")?;
        }
        Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Protocol {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkType {
    #[default]
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

#[inline(always)]
fn parse_time_seconds<Integer: std::str::FromStr>(ss: &str) -> Result<Integer> {
    ss.parse().map_err(|_| Error::TimeInvalid {
        time: ss.to_string(),
    })
}

#[inline(always)]
fn parse_time<Integer: std::str::FromStr + std::ops::Mul<Output = Integer> + From<u32>>(
    ts: &str,
) -> Result<Integer> {
    if let Some(seconds) = ts.strip_suffix('s') {
        parse_time_seconds(seconds)
    } else if let Some(minutes) = ts.strip_suffix('m') {
        Ok(parse_time_seconds::<Integer>(minutes)? * Integer::from(60_u32))
    } else if let Some(hours) = ts.strip_suffix('h') {
        Ok(parse_time_seconds::<Integer>(hours)? * Integer::from(3600_u32))
    } else if let Some(days) = ts.strip_suffix('d') {
        Ok(parse_time_seconds::<Integer>(days)? * Integer::from(86400_u32))
    } else {
        parse_time_seconds(ts)
    }
}
