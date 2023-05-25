use crate::time_utils::convert_time_to_unix_epoch;

/// Represents possible preset time ranges for SDP.
///
/// This is a helper type to make constructing an SDP file more ergonomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Live,
    Playback { start: u64, end: u64 },
}

impl TimeRange {
    /// Create a time range that specifies that the media file is a live stream.
    #[inline]
    pub fn live() -> Self {
        TimeRange::Live
    }

    #[inline]
    /// Create a time range between `start` and `end`.
    ///
    /// # Arguments
    ///
    /// * `start` - Start time of range.
    /// * `end` - End time of range.
    pub fn playback(start: std::time::SystemTime, end: std::time::SystemTime) -> Self {
        TimeRange::Playback {
            start: convert_system_time_to_sdp_time(start),
            end: convert_system_time_to_sdp_time(end),
        }
    }
}

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TimeRange::Live => write!(f, "live"),
            TimeRange::Playback { start, end } => write!(f, "from {start} to {end}"),
        }
    }
}

/// Convert from [`std::time::SystemTime`] to seconds since January 1, 1900 UTC.
#[inline(always)]
pub fn convert_system_time_to_sdp_time(time: std::time::SystemTime) -> u64 {
    convert_time_to_unix_epoch(time) + 2208988800
}
