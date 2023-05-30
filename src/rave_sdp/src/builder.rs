use crate::codec::Parameters as CodecParameters;
use crate::error::{Error, Result};
use crate::sdp::{
    Attribute, Connection, Direction, Kind, Media, MediaItem, Origin, Protocol, Sdp, TimeActive,
};
use crate::time_range::TimeRange;

/// Safe interface for building an SDP session description.
pub struct Builder {
    sdp: Sdp,
    dynamic_payload_type_counter: u8,
}

impl Builder {
    const DYNAMIC_PAYLOAD_TYPE_RANGE: std::ops::Range<u8> = 96..128;

    pub fn new(
        name: &str,
        origin: std::net::IpAddr,
        destination: std::net::IpAddr,
        time_range: TimeRange,
    ) -> Self {
        Self {
            sdp: Sdp {
                version: Default::default(),
                origin: Origin::from(origin),
                session_name: name.to_string(),
                session_description: None,
                uri: None,
                email: None,
                phone: None,
                // FIXME: currently no support for multicast
                connection: Some(Connection::from(destination)),
                bandwidth: Vec::new(),
                time_active: vec![TimeActive::from(time_range)],
                repeats: Vec::new(),
                attributes: Vec::new(),
                media: Vec::new(),
            },
            dynamic_payload_type_counter: Self::DYNAMIC_PAYLOAD_TYPE_RANGE.start,
        }
    }

    #[inline]
    pub fn build(self) -> Sdp {
        self.sdp
    }

    #[inline]
    pub fn set_origin_username(&mut self, username: impl ToString) {
        self.sdp.origin.username = username.to_string();
    }

    #[inline]
    pub fn with_origin_username(mut self, username: impl ToString) -> Self {
        self.set_origin_username(username);
        self
    }

    #[inline]
    pub fn set_description(&mut self, description: impl ToString) {
        self.sdp.session_description = Some(description.to_string());
    }

    #[inline]
    pub fn with_description(mut self, description: impl ToString) -> Self {
        self.set_description(description);
        self
    }

    #[inline]
    pub fn set_uri(&mut self, uri: impl ToString) {
        self.sdp.uri = Some(uri.to_string());
    }

    #[inline]
    pub fn with_uri(mut self, uri: impl ToString) -> Self {
        self.set_uri(uri);
        self
    }

    #[inline]
    pub fn set_email(&mut self, email: impl ToString) {
        self.sdp.email = Some(email.to_string());
    }

    #[inline]
    pub fn with_email(mut self, email: impl ToString) -> Self {
        self.set_email(email);
        self
    }

    #[inline]
    pub fn set_phone(&mut self, phone: impl ToString) {
        self.sdp.phone = Some(phone.to_string());
    }

    #[inline]
    pub fn with_phone(mut self, phone: impl ToString) -> Self {
        self.set_phone(phone);
        self
    }

    #[inline]
    pub fn add_time_active(&mut self, time_range: TimeRange) {
        self.sdp.time_active.push(TimeActive::from(time_range));
    }

    #[inline]
    pub fn with_time_active(mut self, time_range: TimeRange) -> Self {
        self.add_time_active(time_range);
        self
    }

    #[inline]
    pub fn add_property(&mut self, property: impl ToString) {
        self.sdp
            .attributes
            .push(Attribute::Property(property.to_string()));
    }

    #[inline]
    pub fn with_property(mut self, property: impl ToString) -> Self {
        self.add_property(property);
        self
    }

    #[inline]
    pub fn add_value(&mut self, var: impl ToString, val: impl ToString) {
        self.sdp
            .attributes
            .push(Attribute::Value(var.to_string(), val.to_string()))
    }

    #[inline]
    pub fn with_value(mut self, var: impl ToString, val: impl ToString) -> Self {
        self.add_value(var, val);
        self
    }

    #[inline]
    pub fn add_media(
        &mut self,
        kind: Kind,
        title: &str,
        port: u16,
        protocol: Protocol,
        direction: Direction,
        codec_parameters: impl CodecParameters,
    ) -> Result<()> {
        let dynamic_payload_type = self.dynamic_payload_type_counter;
        if !Self::DYNAMIC_PAYLOAD_TYPE_RANGE.contains(&dynamic_payload_type) {
            return Err(Error::TooManyMediaItems);
        }

        self.dynamic_payload_type_counter += 1;

        let mut attributes = codec_parameters.media_attributes(dynamic_payload_type);
        attributes.push(Attribute::Property(direction.to_string()));

        self.sdp.media.push(MediaItem {
            media: Media {
                kind,
                port,
                protocol,
                format: dynamic_payload_type,
            },
            title: Some(title.to_string()),
            connection: None,
            bandwidth: Vec::new(),
            attributes,
        });

        Ok(())
    }

    #[inline]
    pub fn with_media(
        mut self,
        kind: Kind,
        title: &str,
        port: u16,
        protocol: Protocol,
        direction: Direction,
        codec_parameters: impl CodecParameters,
    ) -> Result<Self> {
        self.add_media(kind, title, port, protocol, direction, codec_parameters)?;
        Ok(self)
    }
}
