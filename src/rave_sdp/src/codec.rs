use base64::Engine;

pub use crate::fmt::FMT_RTP_PAYLOAD_DYNAMIC;
pub use crate::sdp::Tag;

pub trait MediaAttributes {
    fn media_attributes(&self) -> Vec<Tag>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecInfo<'params> {
    H264(H264CodecParameters<'params>),
}

impl<'params> CodecInfo<'params> {
    pub fn h264(
        sps: &'params [u8],
        pps: &'params [&'params [u8]],
        packetization_mode: usize,
    ) -> Self {
        Self::H264(H264CodecParameters {
            sps,
            pps,
            packetization_mode,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H264CodecParameters<'params> {
    sps: &'params [u8],
    pps: &'params [&'params [u8]],
    packetization_mode: usize,
}

impl MediaAttributes for CodecInfo<'_> {
    fn media_attributes(&self) -> Vec<Tag> {
        match self {
            CodecInfo::H264(params) => vec![
                h264_rtpmap(),
                h264_fmtp(params.packetization_mode, params.sps, params.pps),
            ],
        }
    }
}

#[inline]
fn h264_rtpmap() -> Tag {
    Tag::Value(
        "rtpmap".to_string(),
        format!("{FMT_RTP_PAYLOAD_DYNAMIC} H264/90000"),
    )
}

fn h264_fmtp(packetization_mode: usize, sps: &[u8], pps: &[&[u8]]) -> Tag {
    let profile_level_id_bytes = &sps[1..4];
    let profile_level_id = profile_level_id_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    let mut parameter_sets = Vec::with_capacity(1 + pps.len());
    parameter_sets.push(base64_encode(sps));
    parameter_sets.extend(pps.iter().map(|item| base64_encode(item)));
    let sprop_parameter_sets = parameter_sets.join(",");

    Tag::Value(
        "fmtp".to_string(),
        format!(
            "{FMT_RTP_PAYLOAD_DYNAMIC} packetization-mode={packetization_mode}; \
                profile-level-id={profile_level_id}; \
                sprop-parameter-sets={sprop_parameter_sets}",
        ),
    )
}

#[inline(always)]
fn base64_encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}
