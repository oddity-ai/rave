use base64::Engine;

use crate::codec::Parameters as ParametersTrait;
use crate::format::FMT_RTP_PAYLOAD_DYNAMIC;
use crate::sdp::Attribute;

// TODO: parse codec info from media attributes.

/// Holds H264 codec-specific parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameters<'params> {
    sps: &'params [u8],
    pps: &'params [&'params [u8]],
    packetization_mode: usize,
}

impl<'params> Parameters<'params> {
    /// Initialize codec-specific information for a H264 stream.
    ///
    /// # Arguments
    ///
    /// * `sps` - Sequence parameter set.
    /// * `pps` - Picture parameter set.
    /// * `packetization_mode` - Packetization mode used by sender.
    pub fn new(
        sps: &'params [u8],
        pps: &'params [&'params [u8]],
        packetization_mode: usize,
    ) -> Self {
        Self {
            sps,
            pps,
            packetization_mode,
        }
    }

    /// Generate `rtpmap` attribute.
    ///
    /// This will generate an RTP map that maps H264 to the dynamic payload identifier 96.
    ///
    /// # Return value
    ///
    /// `rtpmap` attribute for SDP.
    #[inline]
    fn rtpmap_attribute() -> Attribute {
        Attribute::Value(
            "rtpmap".to_string(),
            format!("{FMT_RTP_PAYLOAD_DYNAMIC} H264/90000"),
        )
    }

    /// Generate `fmtp` attribute with H264 stream metadata.
    ///
    /// This will generate a `fmtp` attribute that contains the packetization mode, profile level
    /// ID, and parameter sets. The latter two are extracted from the provided sequence parameter
    /// set and picture parameter sets. It is mapped against the dynamic payload ID 96.
    ///
    /// # Return value
    ///
    /// `fmtp` attribute for SDP.
    fn fmtp_attribute(&self) -> Attribute {
        let profile_level_id_bytes = &self.sps[1..4];
        let profile_level_id = profile_level_id_bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();

        let mut parameter_sets = Vec::with_capacity(1 + self.pps.len());
        parameter_sets.push(base64_encode(self.sps));
        parameter_sets.extend(self.pps.iter().map(|item| base64_encode(item)));

        let sprop_parameter_sets = parameter_sets.join(",");

        Attribute::Value(
            "fmtp".to_string(),
            format!(
                "{} packetization-mode={}; profile-level-id={}; sprop-parameter-sets={}",
                FMT_RTP_PAYLOAD_DYNAMIC,
                self.packetization_mode,
                profile_level_id,
                sprop_parameter_sets,
            ),
        )
    }
}

impl ParametersTrait for Parameters<'_> {
    /// Retrieve corresponding media attributes.
    ///
    /// These attributes are added to the media item to signal media information to the receiver of
    /// the SDP file.
    ///
    /// # Return value
    ///
    /// One or more media attributes.
    fn media_attributes(&self) -> Vec<Attribute> {
        vec![Self::rtpmap_attribute(), self.fmtp_attribute()]
    }
}

#[inline(always)]
fn base64_encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}
