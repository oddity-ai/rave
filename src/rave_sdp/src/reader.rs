use crate::codec::Parameters as CodecParameters;
use crate::error::{Error, Result};
use crate::sdp::{
    AddressType, Attribute, Bandwidth, Connection, Direction, Kind, Media, MediaItem, Origin,
    Protocol, Repeat, Sdp, TimeActive,
};
use crate::time_range::TimeRange;

/// Safe interface to reading an SDP session description.
pub struct Reader {
    sdp: Sdp,
}

impl Reader {
    #[inline]
    pub fn parse(s: &str) -> Result<Self> {
        Ok(Self {
            sdp: Sdp::parse(s)?,
        })
    }

    #[inline]
    pub fn username(&self) -> &str {
        &self.sdp.origin.username
    }

    #[inline]
    pub fn session_id(&self) -> &str {
        &self.sdp.origin.session_id
    }

    #[inline]
    pub fn origin(&self) -> Result<std::net::IpAddr> {
        self.sdp
            .origin
            .unicast_address
            .parse()
            .map_err(|_| Error::OriginUnicastAddressInvalid {
                unicast_address: self.sdp.origin.unicast_address.to_string(),
            })
    }

    #[inline]
    pub fn session_name(&self) -> &str {
        &self.sdp.session_name
    }

    #[inline]
    pub fn session_description(&self) -> Option<&str> {
        self.sdp.session_description.as_deref()
    }

    #[inline]
    pub fn uri(&self) -> Option<&str> {
        self.sdp.uri.as_deref()
    }

    #[inline]
    pub fn email(&self) -> Option<&str> {
        self.sdp.email.as_deref()
    }

    #[inline]
    pub fn phone(&self) -> Option<&str> {
        self.sdp.phone.as_deref()
    }

    #[inline]
    pub fn target(&self) -> Option<()> {
        // TODO: ...
        todo!()
    }

    #[inline]
    pub fn bandwidth_info(&self) -> &[Bandwidth] {
        &self.sdp.bandwidth
    }

    #[inline]
    pub fn time_active(&self) -> TimeRange {
        todo!()
    }

    #[inline]
    pub fn repeats(&self) -> &[Repeat] {
        &self.sdp.repeats
    }

    #[inline]
    pub fn property(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn value(&self, var: &str) -> &str {
        todo!()
    }

    // TODO: API to retrieve media items, resolve relevant information per media item
}

impl std::str::FromStr for Reader {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self> {
        Reader::parse(s)
    }
}
