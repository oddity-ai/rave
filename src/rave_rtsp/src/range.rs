use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub start: Option<NptTime>,
    pub end: Option<NptTime>,
}

impl Range {
    const SUPPORTED_UNITS: [&'static str; 1] = ["npt"];

    pub fn new(start: NptTime, end: NptTime) -> Range {
        Range {
            start: Some(start),
            end: Some(end),
        }
    }

    pub fn new_for_live() -> Range {
        Range {
            start: Some(NptTime::Now),
            end: None,
        }
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "npt=")?;
        match (self.start.as_ref(), self.end.as_ref()) {
            (Some(start), Some(end)) => write!(f, "{start}-{end}"),
            (Some(start), None) => write!(f, "{start}-"),
            (None, Some(end)) => write!(f, "-{end}"),
            (None, None) => write!(f, "-"),
        }
    }
}

impl std::str::FromStr for Range {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(';') {
            None => {
                if let Some((unit, value)) = s.split_once('=') {
                    if Self::SUPPORTED_UNITS.contains(&unit) {
                        if let Some((start, end)) = value.split_once('-') {
                            let start = if !start.is_empty() {
                                Some(start.parse()?)
                            } else {
                                None
                            };
                            let end = if !end.is_empty() {
                                Some(end.parse()?)
                            } else {
                                None
                            };
                            Ok(Range { start, end })
                        } else {
                            Err(Error::RangeMalformed {
                                value: s.to_string(),
                            })
                        }
                    } else {
                        Err(Error::RangeUnitNotSupported {
                            value: s.to_string(),
                        })
                    }
                } else {
                    Err(Error::RangeMalformed {
                        value: s.to_string(),
                    })
                }
            }
            Some((_, time)) => {
                if time.starts_with("time=") {
                    Err(Error::RangeTimeNotSupported {
                        value: s.to_string(),
                    })
                } else {
                    Err(Error::RangeMalformed {
                        value: s.to_string(),
                    })
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NptTime {
    Now,
    Time(f64),
}

impl std::fmt::Display for NptTime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NptTime::Now => write!(f, "now"),
            NptTime::Time(seconds) => write!(f, "{seconds:.3}"),
        }
    }
}

impl std::str::FromStr for NptTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "now" => Ok(NptTime::Now),
            s => match s.split(':').collect::<Vec<_>>().as_slice() {
                [npt_time] => {
                    let npt_time =
                        npt_time
                            .parse::<f64>()
                            .map_err(|_| Error::RangeNptTimeMalfored {
                                value: s.to_string(),
                            })?;
                    Ok(NptTime::Time(npt_time))
                }
                [npt_hh, npt_mm, npt_ss] => {
                    let npt_hh = npt_hh.parse::<u32>();
                    let npt_mm = npt_mm.parse::<u32>();
                    let npt_secs = npt_ss.parse::<f32>();
                    match (npt_hh, npt_mm, npt_secs) {
                        (Ok(hh), Ok(mm), Ok(secs)) => {
                            let npt_time =
                                ((hh * 3600) as f64) + ((mm * 60) as f64) + (secs as f64);
                            Ok(NptTime::Time(npt_time))
                        }
                        _ => Err(Error::RangeNptTimeMalfored {
                            value: s.to_string(),
                        }),
                    }
                }
                _ => Err(Error::RangeNptTimeMalfored {
                    value: s.to_string(),
                }),
            },
        }
    }
}
