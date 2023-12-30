use std::fmt;
use std::ops::{Add, Sub};
use std::time::Duration;

use chrono::{NaiveTime, Timelike};
use serde::Serialize;

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Serialize)]
pub struct Time {
    pub as_string: String,
    #[serde(skip_serializing)]
    pub as_native: NaiveTime,
    #[serde(skip_serializing)]
    pub as_secs: i64,
}

impl Time {
    pub fn compute_offset(&self, other: &str) -> i64 {
        let seconds = Self::from(other.to_string());

        let seconds = if other.contains('-') {
            self - &seconds
        } else {
            self + &seconds
        };

        seconds.as_secs
    }

    fn add_or_subtract(
        &self,
        other: &Time,
        operation: fn(chrono::Duration, chrono::Duration) -> chrono::Duration,
    ) -> Time {
        let time1 = self.as_secs;
        let time2 = other.as_secs;

        let duration1 = chrono::Duration::seconds(time1);
        let duration2 = chrono::Duration::seconds(time2);
        let result_duration = operation(duration1, duration2);

        let result_time = NaiveTime::from_num_seconds_from_midnight_opt(
            u32::try_from(result_duration.num_seconds())
                .expect("Overflow error"),
            0,
        )
        .expect("Invalid time");

        Time::from(result_time.format("%H:%M:%S").to_string())
    }
}

impl Add for &Time {
    type Output = Time;

    fn add(self, other: &Time) -> Time {
        self.add_or_subtract(other, |t1, t2| t1 + t2)
    }
}

impl Sub for &Time {
    type Output = Time;

    fn sub(self, other: &Time) -> Time {
        self.add_or_subtract(other, |t1, t2| t1 - t2)
    }
}

impl From<Duration> for Time {
    fn from(duration: Duration) -> Self {
        Self::from(duration.as_secs())
    }
}

impl From<u64> for Time {
    fn from(duration: u64) -> Self {
        Time::from(format!(
            "{:02}:{:02}:{:02}",
            duration / 3600,
            (duration % 3600) / 60,
            duration % 60
        ))
    }
}

impl From<String> for Time {
    fn from(input: String) -> Self {
        let cleansed: String = input
            .chars()
            .filter(|c| c.is_numeric() || *c == ':')
            .collect();

        let parts: Vec<u8> = cleansed
            .split(':')
            .map(|n| n.parse::<u8>().unwrap_or(0))
            .collect();

        let as_string = match parts.len() {
            1 => format!("00:00:{:02}", parts[0]),
            2 => format!("00:{:02}:{:02}", parts[0], parts[1]),
            3 => format!("{:02}:{:02}:{:02}", parts[0], parts[1], parts[2]),
            _ => String::from("Invalid"),
        };

        let as_native_time = NaiveTime::parse_from_str(&as_string, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(&as_string, "%M:%S"))
            .or_else(|_| NaiveTime::parse_from_str("00:01", "%M:%S"))
            .unwrap();
        let as_secs = i64::from(as_native_time.num_seconds_from_midnight());

        Time {
            as_string,
            as_native: as_native_time,
            as_secs,
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string)
    }
}

pub struct HumanReadableDuration(core::time::Duration);

impl From<Duration> for HumanReadableDuration {
    fn from(duration: Duration) -> Self {
        HumanReadableDuration(duration)
    }
}

impl ToString for HumanReadableDuration {
    fn to_string(&self) -> String {
        let total_seconds = self.0.as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        format!("{days} days, {hours}:{minutes:02}:{seconds:02}")
    }
}

#[derive(Serialize)]
pub struct Track {
    pub elapsed: Time,
    pub total: Time,
}

impl From<Option<(Duration, Duration)>> for Track {
    fn from(time: Option<(Duration, Duration)>) -> Self {
        match time {
            Some((elapsed, total)) => Track {
                elapsed: Time::from(elapsed),
                total: Time::from(total),
            },
            None => Track {
                elapsed: Time::from(0),
                total: Time::from(0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn compute_offset_positive() {
        let time = Time::from("0:05".to_string());

        assert_eq!(time.compute_offset("0:15"), 20);
    }

    #[test]
    fn compute_offset_negative() {
        let time = Time::from("1:00".to_string());

        assert_eq!(time.compute_offset("-0:30"), 30);
    }

    #[test]
    fn test_time_from_duration() {
        let duration = Duration::from_secs(3661); // 1 hour, 1 minute, 1 second
        let time = Time::from(duration);
        assert_eq!(time.as_string, "01:01:01");
    }

    #[test]
    fn test_time_from_u64() {
        let duration: u64 = 3661; // 1 hour, 1 minute, 1 second
        let time = Time::from(duration);
        assert_eq!(time.as_string, "01:01:01");
    }

    #[test]
    fn test_time_display() {
        let time = Time::from("5:30".to_string());
        assert_eq!(format!("{time}"), "00:05:30");
    }

    #[test]
    fn test_human_readable_duration_to_string() {
        let duration = Duration::from_secs(90061); // 1 day, 1 hour, 1 minute, 1 second
        let human_duration = HumanReadableDuration::from(duration);
        assert_eq!(human_duration.to_string(), "1 days, 1:01:01");
    }

    #[test]
    fn test_track_from_option() {
        let elapsed = Duration::from_secs(60);
        let total = Duration::from_secs(300);
        let track = Track::from(Some((elapsed, total)));
        assert_eq!(track.elapsed.as_string, "00:01:00");
        assert_eq!(track.total.as_string, "00:05:00");
    }
}
