use std::fmt;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use eyre::WrapErr;
use serde::Serialize;

use crate::args::{OnOff, OutputFormat};

#[derive(Serialize)]
pub struct Status {
    volume: String,
    state: String,
    artist: String,
    title: String,
    position: u32,
    queue_count: u32,
    elapsed: Time,
    track_length: Time,
    repeat: OnOff,
    random: OnOff,
    single: OnOff,
    consume: OnOff,
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

#[derive(Serialize)]
struct Song(mpd::song::Song);

#[derive(Serialize)]
struct Current {
    artist: String,
    title: String,
}

impl From<Status> for Current {
    fn from(status: Status) -> Self {
        Current {
            artist: status.artist,
            title: status.title,
        }
    }
}

impl From<Song> for Current {
    fn from(song: Song) -> Self {
        Current {
            artist: song.0.artist.unwrap_or("".to_string()),
            title: song.0.title.unwrap_or("".to_string()),
        }
    }
}

impl fmt::Display for Current {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "artist={}\ntitle={}", self.artist, self.title)
    }
}

#[derive(Serialize)]
pub struct TrackTime {
    elapsed: Time,
    total: Time,
}

impl From<Option<(Duration, Duration)>> for TrackTime {
    fn from(time: Option<(Duration, Duration)>) -> Self {
        match time {
            Some((elapsed, total)) => TrackTime {
                elapsed: Time::from(elapsed),
                total: Time::from(total),
            },
            None => TrackTime {
                elapsed: Time::from(0),
                total: Time::from(0),
            },
        }
    }
}

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

struct HumanReadableDuration(core::time::Duration);

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

        format!("{} days, {}:{:02}:{:02}", days, hours, minutes, seconds)
    }
}
#[derive(Serialize)]
struct Stats {
    artists: u32,
    albums: u32,
    songs: u32,
    uptime: String,
    playtime: String,
    db_playtime: String,
    db_update: String,
}

impl Stats {
    pub fn new(stats: mpd::stats::Stats) -> Self {
        let db_update =
            match Utc.timestamp_opt(stats.db_update.as_secs() as i64, 0) {
                chrono::LocalResult::Single(date_time) => {
                    date_time.format("%a %b %d %H:%M:%S %Y").to_string()
                }
                _ => "".to_string(),
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
pub struct Versions {
    mpd: String,
    mp_cli: String,
}

impl fmt::Display for Versions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mpd={}\nmp-cli={}", self.mpd, self.mp_cli)
    }
}

pub struct Client {
    client: mpd::Client,
    format: OutputFormat,
}

impl Client {
    pub fn new(
        bind_to_address: &str,
        port: &str,
        format: OutputFormat,
    ) -> eyre::Result<Client> {
        // TODO: read connection details from mpd.conf
        let client = mpd::Client::connect(format!("{bind_to_address}:{port}"))
            .wrap_err("Error connecting to mpd server".to_string())?;

        Ok(Self { client, format })
    }

    //
    // queue related commands
    //
    pub fn crop(&mut self) -> eyre::Result<Option<String>> {
        // determine current song position
        // remove all songs before current song
        // remove all songs from 1 onwards
        let status = self.status()?;
        let current_position = status.position;
        let length = status.queue_count;

        if length < 1 {
            return self.current_status();
        }

        self.client.delete(0..current_position)?;
        // it doesn't matter that the range is out of bounds
        self.client.delete(1..length)?;

        self.current_status()
    }

    //
    // playback related commands
    //
    pub fn current(&mut self) -> eyre::Result<Option<String>> {
        let current = Current::from(self.status()?);

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&current)?,
            OutputFormat::Text => current.to_string(),
        };

        Ok(Some(response))
    }

    pub fn play(&mut self) -> eyre::Result<Option<String>> {
        self.client.play()?;

        self.current_status()
    }

    pub fn next(&mut self) -> eyre::Result<Option<String>> {
        self.client.next()?;

        self.current_status()
    }

    pub fn prev(&mut self) -> eyre::Result<Option<String>> {
        self.client.prev()?;

        self.current_status()
    }

    pub fn pause(&mut self) -> eyre::Result<Option<String>> {
        self.client.pause(true)?;

        self.current_status()
    }

    pub fn pause_if_playing(&mut self) -> eyre::Result<Option<String>> {
        match self.client.status()?.state {
            mpd::State::Play => self.pause(),
            mpd::State::Pause => Err(eyre::eyre!("")),
            mpd::State::Stop => Err(eyre::eyre!("")),
        }
    }

    pub fn cdprev(&mut self) -> eyre::Result<Option<String>> {
        let default_duration = Duration::from_secs(0);
        let status = &self.client.status()?;
        let current = status.elapsed.unwrap_or(default_duration).as_secs();

        if current < 3 {
            self.prev()
        } else {
            let place = match status.song {
                Some(ref song) => song.pos,
                None => 0,
            };
            self.client.seek(place, 0)?;

            self.current_status()
        }
    }

    pub fn toggle(&mut self) -> eyre::Result<Option<String>> {
        match self.client.status()?.state {
            mpd::State::Play => self.pause(),
            mpd::State::Pause => self.play(),
            mpd::State::Stop => self.play(),
        }
    }

    pub fn stop(&mut self) -> eyre::Result<Option<String>> {
        self.client.stop()?;

        self.current_status()
    }

    //
    // playlist related commands
    //

    pub fn clear(&mut self) -> eyre::Result<Option<String>> {
        self.client.clear()?;

        self.current_status()
    }

    pub fn queued(&mut self) -> eyre::Result<Option<String>> {
        if let Some(song) =
            self.client.queue().map_err(|e| eyre::eyre!(e))?.get(0)
        {
            // safe to unwrap because we know we have a song
            let current = Current::from(Song(song.clone()));

            let response = match self.format {
                OutputFormat::Json => serde_json::to_string(&current)?,
                OutputFormat::Text => current.to_string(),
            };

            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    pub fn shuffle(&mut self) -> eyre::Result<Option<String>> {
        self.client.shuffle(..)?;

        self.current_status()
    }

    pub fn repeat(
        &mut self,
        state: Option<OnOff>,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.repeat,
        };

        self.client.repeat(state)?;

        self.current_status()
    }

    pub(crate) fn random(
        &mut self,
        state: Option<OnOff>,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.random,
        };

        self.client.random(state)?;

        self.current_status()
    }

    pub(crate) fn single(
        &mut self,
        state: Option<OnOff>,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.single,
        };

        self.client.single(state)?;

        self.current_status()
    }

    pub fn consume(
        &mut self,
        state: Option<OnOff>,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.consume,
        };

        self.client.consume(state)?;

        self.current_status()
    }

    pub fn version(&mut self) -> eyre::Result<Option<String>> {
        let mpd = format!(
            "{}.{}.{}",
            self.client.version.0, self.client.version.1, self.client.version.2
        );
        let mp_cli = env!("CARGO_PKG_VERSION").to_string();

        let versions = Versions { mpd, mp_cli };

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&versions)?,
            OutputFormat::Text => versions.to_string(),
        };

        Ok(Some(response))
    }

    pub fn stats(&mut self) -> eyre::Result<Option<String>> {
        let stats = Stats::new(self.client.stats()?);

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&stats)?,
            OutputFormat::Text => stats.to_string(),
        };

        Ok(Some(response))
    }

    //
    // volume related commands
    //

    pub fn set_volume(&mut self, input: &str) -> eyre::Result<Option<String>> {
        let current = self.client.status()?.volume;

        let target = match input {
            matched if matched.starts_with('+') => {
                if let Ok(volume) = matched[1..].parse::<i8>() {
                    current.checked_add(volume).unwrap_or(100).min(100)
                } else {
                    panic!("Invalid volume increment, must be between 1-100")
                }
            }
            matched if matched.starts_with('-') => {
                if let Ok(volume) = matched[1..].parse::<i8>() {
                    current.checked_sub(volume).unwrap_or(100).max(0)
                } else {
                    panic!("Invalid volume increment, must be between 1-100")
                }
            }
            _ => input.parse::<i8>().unwrap_or(0),
        };

        self.client
            .volume(target)
            .map(|_| None)
            .map_err(eyre::Report::from)
    }

    //
    // output related commands
    //

    fn status(&mut self) -> eyre::Result<Status> {
        let status = self.client.status()?;

        let volume = status.volume.to_string();

        let current_song = self.client.currentsong()?;

        let artist = current_song
            .as_ref()
            .and_then(|song| song.artist.as_ref())
            .map_or("".to_string(), ToString::to_string);
        let title = current_song
            .as_ref()
            .and_then(|song| song.title.as_ref())
            .map_or("".to_string(), ToString::to_string);

        let state = match status.state {
            mpd::State::Play => "play",
            mpd::State::Pause => "pause",
            mpd::State::Stop => "stop",
        }
        .to_string();

        let position = match status.song {
            Some(song) => song.pos,
            None => 0,
        };
        let time = TrackTime::from(status.time);

        Ok(Status {
            volume,
            state,
            artist,
            title,
            position,
            queue_count: status.queue_len,
            elapsed: time.elapsed,
            track_length: time.total,
            repeat: OnOff::from(status.repeat),
            random: OnOff::from(status.random),
            single: OnOff::from(status.single),
            consume: OnOff::from(status.consume),
        })
    }

    pub fn current_status(&mut self) -> eyre::Result<Option<String>> {
        let status = self.status()?;
        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&status)?,
            OutputFormat::Text => format!("{}", status),
        };

        Ok(Some(response))
    }
}
