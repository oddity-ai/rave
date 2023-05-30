pub mod h264;

/// Codec parameters.
///
/// This is implemented by all types that represent codec-specific parameters.
pub trait Parameters {
    /// Retrieve corresponding media attributes.
    ///
    /// These attributes are added to the media item to signal media information to the receiver of
    /// the SDP file.
    ///
    /// # Arguments
    ///
    /// * `dynamic_payload_type` - Dynamic payload type to associate with media item.
    ///
    /// # Return value
    ///
    /// One or more media attributes.
    fn media_attributes(&self, dynamic_payload_type: u8) -> Vec<crate::sdp::Attribute>;
}
