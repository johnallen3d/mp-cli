use std::fmt;

use serde::Serialize;

use crate::se::serialize_time;
use crate::{time::Time, OnOff};

#[derive(Debug, Serialize, PartialEq)]
pub enum State {
    Stop,
    Play,
    Pause,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = match self {
            State::Stop => "stop",
            State::Play => "play",
            State::Pause => "pause",
        };
        write!(f, "{state}")
    }
}

impl From<mpd::State> for State {
    fn from(state: mpd::State) -> Self {
        match state {
            mpd::State::Stop => State::Stop,
            mpd::State::Play => State::Play,
            mpd::State::Pause => State::Pause,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Status {
    pub volume: String,
    pub state: State,
    pub artist: String,
    pub album: String,
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
    pub file_path: Option<String>,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "volume={}\nstate={}\nartist={}\nalbum={}\ntitle={}\nposition={}\nqueue_count={}\nelapsed={}\ntrack_length={}\nrepeat={}\nrandom={}\nsingle={}\nconsume={}",
            self.volume,
            self.state,
            self.artist,
            self.album,
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
    use crate::time::Time;
    use crate::OnOff;

    #[test]
    fn test_status_display_format() {
        let status = Status {
            volume: "100".to_string(),
            state: State::Play,
            artist: "Phish".to_string(),
            album: "A Picture Of Nectar".to_string(),
            title: "Chalk Dust Torture".to_string(),
            position: 3,
            queue_count: 10,
            elapsed: Time::from(60),
            track_length: Time::from(300),
            repeat: OnOff::Off,
            random: OnOff::On,
            single: OnOff::Off,
            consume: OnOff::Off,
            file_path: Some("path/to/file".to_string()),
        };

        let display_output = format!("{status}");
        let expected_output = "volume=100\nstate=play\nartist=Phish\nalbum=A Picture Of Nectar\ntitle=Chalk Dust Torture\nposition=3\nqueue_count=10\nelapsed=00:01:00\ntrack_length=00:05:00\nrepeat=off\nrandom=on\nsingle=off\nconsume=off";

        assert_eq!(display_output, expected_output);
    }
}
