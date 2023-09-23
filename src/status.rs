use std::fmt;

use serde::Serialize;

use crate::{args::OnOff, time::Time};

#[derive(Serialize)]
pub struct Status {
    pub volume: String,
    pub state: String,
    pub artist: String,
    pub title: String,
    pub position: u32,
    pub queue_count: u32,
    pub elapsed: Time,
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
