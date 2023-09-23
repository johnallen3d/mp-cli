use std::fmt;
use std::time::Duration;

use serde::Serialize;

#[derive(Serialize)]
pub struct Time(String);

impl From<Duration> for Time {
    fn from(duration: Duration) -> Self {
        Time(format!(
            "{:02}:{:02}",
            duration.as_secs() / 60,
            duration.as_secs() % 60
        ))
    }
}

impl From<u32> for Time {
    fn from(duration: u32) -> Self {
        Time(format!("{:02}:{:02}", duration / 60, duration % 60))
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
    fn test_time_from_duration() {
        let duration = Duration::from_secs(3661); // 1 hour, 1 minute, 1 second
        let time = Time::from(duration);
        assert_eq!(time.0, "61:01");
    }

    #[test]
    fn test_time_from_u32() {
        let duration: u32 = 3661; // 1 hour, 1 minute, 1 second
        let time = Time::from(duration);
        assert_eq!(time.0, "61:01");
    }

    #[test]
    fn test_time_display() {
        let time = Time("61:01".to_string());
        assert_eq!(format!("{time}"), "61:01");
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
        assert_eq!(track.elapsed.0, "01:00");
        assert_eq!(track.total.0, "05:00");
    }
}
