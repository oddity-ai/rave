use crate::codec::Parameters as CodecParameters;
use crate::format::FMT_RTP_PAYLOAD_DYNAMIC;
use crate::sdp::{Connection, MediaItem, Origin, Sdp, TimeActive};
use crate::time_range::TimeRange;

pub struct Builder {
    sdp: Sdp,
    // TODO: fmt mapping state
}

impl Builder {
    /// Create new simple media session description.
    ///
    /// # Arguments
    ///
    /// * `origin` - Origin of media session.
    /// * `name` - Name of media.
    /// * `destination` - Destination of media session.
    /// * `time_range` - Time range of media session.
    pub fn new(
        origin: std::net::IpAddr,
        name: &str,
        destination: std::net::IpAddr,
        time_range: TimeRange,
    ) -> Self {
        // TODO: maybe we do this differently ?
        Self {
            sdp: Sdp {
                version: Default::default(),
                origin: Origin::from(origin),
                session_name: name.to_string(),
                session_description: None,
                uri: None,
                email: None,
                phone: None,
                connection: Some(Connection::from(destination)),
                bandwidth: Vec::new(),
                time_active: vec![TimeActive::from(time_range)],
                repeats: Vec::new(),
                attributes: Vec::new(),
                media: Vec::new(),
            },
        }
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

    pub fn with_attribute(mut self, attribute: Attribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = Attribute>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    /// Add a media entry to the media session.
    ///
    /// # Arguments
    ///
    /// * `kind` - Kind of media to add.
    /// * `title` - Title of media to add.
    /// * `port` - Communication port on which media is available.
    /// * `protocol` - Protocol over which media is transmitted.
    /// * `direction` - Direction in which media flows.
    /// * `codec_parameters` - Codec-specific parameters.
    pub fn with_media(
        mut self,
        kind: Kind,
        title: &str,
        port: u16,
        protocol: Protocol,
        direction: Direction,
        codec_parameters: impl CodecParameters,
    ) -> Self {
        let mut attributes = codec_parameters.media_attributes();
        attributes.push(Attribute::Property(direction.to_string()));

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
            attributes,
        });
        self
    }
}
