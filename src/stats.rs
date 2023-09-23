use std::fmt;

use chrono::{TimeZone, Utc};
use serde::Serialize;

use crate::time::HumanReadableDuration;

#[derive(Serialize)]
pub struct Stats {
    pub artists: u32,
    pub albums: u32,
    pub songs: u32,
    pub uptime: String,
    pub playtime: String,
    pub db_playtime: String,
    pub db_update: String,
}

impl Stats {
    pub fn new(stats: mpd::stats::Stats) -> Self {
        let seconds: i64 =
            stats.db_update.as_secs().try_into().unwrap_or(i64::MAX);
        let db_update = match Utc.timestamp_opt(seconds, 0) {
            chrono::LocalResult::Single(date_time) => {
                date_time.format("%a %b %d %H:%M:%S %Y").to_string()
            }
            _ => String::new(),
        };

        Self {
            artists: stats.artists,
            albums: stats.albums,
            songs: stats.songs,
            uptime: HumanReadableDuration::from(stats.uptime).to_string(),
            playtime: HumanReadableDuration::from(stats.playtime).to_string(),
            db_playtime: HumanReadableDuration::from(stats.db_playtime)
                .to_string(),
            db_update,
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "artists={}\nalbums={}\nsongs={}\nuptime={}\nplaytime={}\ndb_playtime={}\ndb_update={}",
            self.artists,
            self.albums,
            self.songs,
            self.uptime,
            self.playtime,
            self.db_playtime,
            self.db_update,
            )
    }
}

#[derive(Serialize)]
pub struct Outputs {
    pub outputs: Vec<Output>,
}

#[derive(Serialize)]
pub struct Output(String);

impl From<String> for Output {
    fn from(name: String) -> Self {
        Output(name)
    }
}

impl fmt::Display for Outputs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, output) in self.outputs.iter().enumerate() {
            write!(f, "{}={}", index, output.0)?;
        }

        Ok(())
    }
}
