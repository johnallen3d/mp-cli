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
pub struct Output {
    pub id: u32,
    pub name: String,
    pub enabled: Enabled,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum Enabled {
    Enabled,
    Disabled,
}

impl From<bool> for Enabled {
    fn from(value: bool) -> Self {
        if value {
            Enabled::Enabled
        } else {
            Enabled::Disabled
        }
    }
}

impl fmt::Display for Enabled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Enabled::Enabled => write!(f, "enabled"),
            Enabled::Disabled => write!(f, "disabled"),
        }
    }
}

impl From<mpd::output::Output> for Output {
    fn from(inner: mpd::output::Output) -> Self {
        Output {
            id: inner.id,
            name: inner.name,
            enabled: Enabled::from(inner.enabled),
        }
    }
}

impl fmt::Display for Outputs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for output in &self.outputs {
            write!(f, "{}={} ({})", output.id, output.name, output.enabled)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_stats_creation() {
        let mpd_stats = mpd::stats::Stats {
            artists: 100,
            albums: 200,
            songs: 300,
            uptime: Duration::new(1000, 0),
            playtime: Duration::new(5000, 0),
            db_playtime: Duration::new(6000, 0),
            db_update: Duration::new(1_000_000, 0),
        };

        let stats = Stats::new(mpd_stats);

        assert_eq!(stats.artists, 100);
        assert_eq!(stats.albums, 200);
        assert_eq!(stats.songs, 300);
        assert_eq!(stats.uptime, "0 days, 0:16:40");
        assert_eq!(stats.playtime, "0 days, 1:23:20");
        assert_eq!(stats.db_playtime, "0 days, 1:40:00");

        assert!(!stats.db_update.is_empty());
    }
}
