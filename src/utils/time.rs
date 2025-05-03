use std::{str::FromStr, time::{Duration, Instant, SystemTime}};
use humantime::{format_duration, format_rfc3339, parse_duration, Timestamp};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleTime{
    Delay(Duration),
    At(SystemTime)
}
#[derive(Debug, Error)]
pub enum ScheduleTimeError {
    #[error("Invalid format: expected 'delay:<duration>' or 'at:<timestamp>'")]
    InvalidFormat,
    #[error("Unknown schedule type tag: '{0}'")]
    UnknownTag(String),
    #[error("Failed to parse duration: {0}")]
    DurationParseError(#[from] humantime::DurationError),
    #[error("Failed to parse timestamp: {0}")]
    TimestampParseError(#[from] humantime::TimestampError),
}



impl FromStr for ScheduleTime {
    type Err = ScheduleTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(ScheduleTimeError::InvalidFormat);
        }

        let tag = parts[0].trim().to_lowercase();
        let value_str = parts[1].trim();

        match tag.as_str() {
            "delay" => {
                let duration = parse_duration(value_str)?;
                Ok(ScheduleTime::Delay(duration))
            }
            "at" => {
                // humantime::parse_rfc3339 also works, but Timestamp::from_str
                // provides a consistent parsing interface provided by humantime
                let timestamp = Timestamp::from_str(value_str)?;
                 Ok(ScheduleTime::At(timestamp.into())) // Convert humantime::Timestamp to std::time::SystemTime
            }
            _ => Err(ScheduleTimeError::UnknownTag(tag)),
        }
    }
}

use std::fmt;

impl fmt::Display for ScheduleTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleTime::Delay(duration) => {
                write!(f, "delay:{}", format_duration(*duration))
            }
            ScheduleTime::At(system_time) => {
                 // Use humantime's formatter for SystemTime to RFC3339
                write!(f, "at:{}", format_rfc3339(*system_time))
            }
        }
    }
}

#[cfg(test)]
mod tests{

    use super::*;
    use crate::utils::time::*;

    #[test]
    fn test_at(){
        let system_time = SystemTime::now();
        let schedule_time = ScheduleTime::At(system_time.clone());
        if let ScheduleTime::At(inner_time) = schedule_time {
            assert_eq!(system_time, inner_time);
        } else {
            panic!("Expected ScheduleTime::At variant.");
        }
    }
}