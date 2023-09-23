use std::fmt;
use std::path::Path;
use std::time::Duration;

use eyre::WrapErr;
use serde::Serialize;

use crate::song::Finder;
use crate::stats::Output;
use crate::stats::Outputs;
use crate::{
    args::{OnOff, OutputFormat},
    song::Current,
    song::Playlist,
    song::Playlists,
    song::Song,
    song::TrackList,
    stats::Stats,
    status::Status,
    time::Track,
};

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
        let client = mpd::Client::connect(format!("{bind_to_address}:{port}"))
            .wrap_err("Error connecting to mpd server".to_string())?;

        Ok(Self { client, format })
    }

    //
    // queue related commands
    //
    pub fn add(&mut self, path: &str) -> eyre::Result<Option<String>> {
        let music_dir = self.client.music_directory()?;

        let absolute_path = if path.starts_with(&music_dir) {
            path.to_string()
        } else {
            format!("{music_dir}/{path}")
        };

        let mut finder = Finder::new(music_dir);

        finder.find(Path::new(Path::new(&absolute_path)))?;

        for file in finder.found {
            let song = crate::mpd::song::Song {
                file: file.relative_path,
                ..Default::default()
            };

            self.client
                .push(song.clone())
                .wrap_err(format!("unkown or inalid path: {}", song.file))?;
        }

        Ok(None)
    }

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

    pub fn del(
        &mut self,
        position: Option<u32>,
    ) -> eyre::Result<Option<String>> {
        let position = match position {
            Some(position) => position,
            None => self.status()?.position,
        };

        self.client.delete(position)?;

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

    pub fn play(
        &mut self,
        position: Option<u32>,
    ) -> eyre::Result<Option<String>> {
        if position.is_none() {
            self.client.play()?;
            return self.current_status();
        }
        // TODO: this is super hacky, can't find a "jump" in rust-mpd

        // pause
        // get current position
        // next/prev to desired position
        // play

        let position = position.unwrap();
        let current_position = self.status()?.position;

        self.pause()?;

        if current_position > position {
            for _ in (position..current_position).rev() {
                self.prev()?;
            }
        } else {
            for _ in (current_position..position).rev() {
                self.next()?;
            }
        }

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
            mpd::State::Pause | mpd::State::Stop => Err(eyre::eyre!("")),
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
            mpd::State::Pause | mpd::State::Stop => self.play(None),
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

    pub fn outputs(&mut self) -> eyre::Result<Option<String>> {
        let outputs = self.client.outputs()?;
        let outputs: Vec<Output> =
            outputs.into_iter().map(|p| Output::from(p.name)).collect();
        let outputs = Outputs { outputs };

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&outputs)?,
            OutputFormat::Text => outputs.to_string(),
        };

        Ok(Some(response))
    }

    pub fn queued(&mut self) -> eyre::Result<Option<String>> {
        if let Some(song) =
            self.client.queue().map_err(|e| eyre::eyre!(e))?.get(0)
        {
            // safe to unwrap because we know we have a song
            let current = Current::from(Song {
                inner: song.clone(),
            });

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

    pub fn lsplaylists(&mut self) -> eyre::Result<Option<String>> {
        let playlists = self.client.playlists()?;
        let playlists: Vec<Playlist> = playlists
            .into_iter()
            .map(|p| Playlist::from(p.name))
            .collect();
        let playlists = Playlists { playlists };

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&playlists)?,
            OutputFormat::Text => playlists.to_string(),
        };

        Ok(Some(response))
    }

    pub fn playlist(
        &mut self,
        name: Option<String>,
    ) -> eyre::Result<Option<String>> {
        // if given a name list songs in that playlist
        // if `None` list songs in current playlist
        let songs = match name {
            Some(name) => self.client.playlist(&name)?,
            None => self.client.queue()?,
        };

        let songs: Vec<Current> = songs
            .into_iter()
            .map(|s| Current::from(Song { inner: s }))
            .collect();
        let track_list = TrackList { songs };

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&track_list)?,
            OutputFormat::Text => track_list.to_string(),
        };

        Ok(Some(response))
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

    pub fn crossfade(
        &mut self,
        seconds: Option<String>,
    ) -> eyre::Result<Option<String>> {
        let crossfade = match seconds {
            Some(secs) => secs.parse::<i64>().wrap_err(format!(
                "\"{secs}\" is not 0 or a positive number"
            ))?,
            None => 0,
        };

        self.client
            .crossfade(crossfade)
            .wrap_err(format!("\"{crossfade}\" is too large"))?;

        Ok(Some(format!("crossfade: {crossfade}")))
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

    pub fn save(&mut self, name: &str) -> eyre::Result<Option<String>> {
        self.client
            .save(name)
            .wrap_err(format!("Playlist already exists: {name}"))?;

        Ok(None)
    }

    pub fn rm(&mut self, name: &str) -> eyre::Result<Option<String>> {
        self.client
            .pl_remove(name)
            .wrap_err(format!("Unknown playlist: {name}"))?;

        Ok(None)
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
            .map_or(String::new(), ToString::to_string);
        let title = current_song
            .as_ref()
            .and_then(|song| song.title.as_ref())
            .map_or(String::new(), ToString::to_string);

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
        let time = Track::from(status.time);

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
            OutputFormat::Text => format!("{status}"),
        };

        Ok(Some(response))
    }
}
