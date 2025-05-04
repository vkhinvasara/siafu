//! Module for parsing and representing schedule times in Siafu.
//!
//! `ScheduleTime` encapsulates either a relative delay (`Delay`) or an absolute system time (`At`).
//! It implements `std::str::FromStr`, accepting strings prefixed with `delay:` or `at:` and parsing them using the `humantime` crate.
//!
//! # Examples
//!
//! ```rust
//! use siafu::utils::time::ScheduleTime;
//! use std::str::FromStr;
//!
//! // Parse a human-friendly delay
//! let delay = ScheduleTime::from_str("delay:1h 30m").unwrap();
//! // Parse an RFC3339 timestamp
//! let at = ScheduleTime::from_str("at:2025-05-05T12:00:00Z").unwrap();
//! ```

use std::{str::FromStr, time::{Duration, SystemTime}};
use humantime::{format_duration, format_rfc3339, parse_duration, Timestamp};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents when a job should run: either a relative delay or an absolute time.
pub enum ScheduleTime{
    /// Run after a specified `Duration`.
    Delay(Duration),
    /// Run at a specific `SystemTime`.
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
    use std::time::Duration;

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

    // Test parsing a human-readable delay and verifying the Duration
    #[test]
    fn test_delay_parsing() {
        let expected = Duration::from_secs(3661);
        let parsed: ScheduleTime = "delay:1h 1m 1s".parse().unwrap();
        match parsed {
            ScheduleTime::Delay(d) => assert_eq!(d, expected),
            _ => panic!("Expected ScheduleTime::Delay variant."),
        }
    }

    // Test parsing and displaying a fixed RFC3339 timestamp
    #[test]
    fn test_at_parsing_and_display() {
        let timestamp_str = "2020-01-01T12:34:56Z";
        let input = format!("at:{}", timestamp_str);
        let parsed: ScheduleTime = input.parse().unwrap();
        match &parsed {
            ScheduleTime::At(_) => {
                let formatted = parsed.to_string();
                assert_eq!(formatted, input);
                // Also ensure the inner SystemTime corresponds to the same rfc3339
                assert_eq!(formatted.strip_prefix("at:").unwrap(), timestamp_str);
            }
            _ => panic!("Expected ScheduleTime::At variant."),
        }
    }

    // Test that formatting a Delay round-trips back to the same string
    #[test]
    fn test_round_trip_delay() {
        let orig = "delay:15m";
        let sched: ScheduleTime = orig.parse().unwrap();
        assert_eq!(sched.to_string(), orig);
    }

    // Test error handling for invalid inputs
    #[test]
    fn test_error_invalid_format() {
        let err = "".parse::<ScheduleTime>().unwrap_err();
        assert!(matches!(err, ScheduleTimeError::InvalidFormat));
    }

    #[test]
    fn test_error_unknown_tag() {
        let err = "foo:10s".parse::<ScheduleTime>().unwrap_err();
        assert!(matches!(err, ScheduleTimeError::UnknownTag(_)));
    }

    #[test]
    fn test_error_duration_parse() {
        let err = "delay:abc".parse::<ScheduleTime>().unwrap_err();
        assert!(matches!(err, ScheduleTimeError::DurationParseError(_)));
    }

    #[test]
    fn test_error_timestamp_parse() {
        let err = "at:abc".parse::<ScheduleTime>().unwrap_err();
        assert!(matches!(err, ScheduleTimeError::TimestampParseError(_)));
    }
}