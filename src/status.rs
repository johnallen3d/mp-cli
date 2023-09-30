use std::fmt;

use serde::Serialize;

use crate::se::serialize_time;
use crate::{args::OnOff, time::Time};

#[derive(Serialize)]
pub struct Status {
    pub volume: String,
    pub state: String,
    pub artist: String,
    pub title: String,
    pub position: u32,
    pub queue_count: u32,
    #[serde(serialize_with = "serialize_time")]
    pub elapsed: Time,
    #[serde(serialize_with = "serialize_time")]
    pub track_length: Time,
    pub repeat: OnOff,
    pub random: OnOff,
    pub single: OnOff,
    pub consume: OnOff,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "volume={}\nstate={}\nartist={}\ntitle={}\nposition={}\nqueue_count={}\nelapsed={}\ntrack_length={}\nrepeat={}\nrandom={}\nsingle={}\nconsume={}",
            self.volume,
            self.state,
            self.artist,
            self.title,
            self.position,
            self.queue_count,
            self.elapsed,
            self.track_length,
            self.repeat,
            self.random,
            self.single,
            self.consume,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::OnOff;
    use crate::time::Time;

    #[test]
    fn test_status_display_format() {
        let status = Status {
            volume: "100".to_string(),
            state: "playing".to_string(),
            artist: "Phish".to_string(),
            title: "Chalk Dust Torture".to_string(),
            position: 3,
            queue_count: 10,
            elapsed: Time::from(60),
            track_length: Time::from(300),
            repeat: OnOff::Off,
            random: OnOff::On,
            single: OnOff::Off,
            consume: OnOff::Off,
        };

        let display_output = format!("{status}");
        let expected_output = "volume=100\nstate=playing\nartist=Phish\ntitle=Chalk Dust Torture\nposition=3\nqueue_count=10\nelapsed=00:01:00\ntrack_length=00:05:00\nrepeat=off\nrandom=on\nsingle=off\nconsume=off";

        assert_eq!(display_output, expected_output);
    }
}
