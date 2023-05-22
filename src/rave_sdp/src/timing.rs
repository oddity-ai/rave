#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Live,
    Playback { start: u64, end: u64 },
}

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TimeRange::Live => write!(f, "live"),
            TimeRange::Playback { start, end } => write!(f, "from {start} to {end}"),
        }
    }
}
