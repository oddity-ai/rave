/// Convert from [`std::time::SystemTime`] to UNIX epoch timestamp.
#[inline]
pub fn convert_time_to_unix_epoch(time: std::time::SystemTime) -> u64 {
    time.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
