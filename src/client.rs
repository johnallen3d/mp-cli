use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use eyre::WrapErr;
use serde::Serialize;

use crate::{
    args::{OnOff, OutputFormat},
    song::Current,
    song::Finder,
    song::Listing,
    song::Playlist,
    song::Playlists,
    song::Song,
    song::TrackList,
    stats::Output,
    stats::Outputs,
    stats::Stats,
    status::Status,
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
            PathBuf::from(&music_dir)
                .join(path)
                .to_str()
                .unwrap()
                .to_string()
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
            outputs.into_iter().map(Output::from).collect();
        let outputs = Outputs { outputs };

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&outputs)?,
            OutputFormat::Text => outputs.to_string(),
        };

        Ok(Some(response))
    }

    fn output_for(&mut self, name_or_id: &str) -> Result<u32, eyre::Error> {
        let id: u32 = if let Ok(parsed_id) = name_or_id.parse::<u32>() {
            parsed_id
        } else {
            self.client
                .outputs()?
                .iter()
                .find(|&o| o.name == name_or_id)
                .ok_or_else(|| eyre::eyre!("unknown output: {}", name_or_id))?
                .id
        };

        Ok(id)
    }

    fn enable_or_disable(
        &mut self,
        enable: bool,
        args: Vec<String>,
    ) -> eyre::Result<Option<String>> {
        let mut only = false;
        let mut outputs = Vec::new();

        for arg in args {
            if arg == "only" {
                only = true;
            } else {
                outputs.push(arg);
            }
        }

        if only {
            // first disable all outputs
            for output in self.client.outputs()? {
                self.client.output(output, enable)?;
            }
        }

        for name_or_id in outputs {
            let id = self.output_for(&name_or_id)?;

            self.client.output(id, enable)?;
        }

        self.outputs()
    }

    pub fn enable(
        &mut self,
        args: Vec<String>,
    ) -> eyre::Result<Option<String>> {
        self.enable_or_disable(true, args)
    }

    pub fn disable(
        &mut self,
        args: Vec<String>,
    ) -> eyre::Result<Option<String>> {
        self.enable_or_disable(false, args)
    }

    pub fn toggle_output(
        &mut self,
        args: Vec<String>,
    ) -> eyre::Result<Option<String>> {
        if args.is_empty() {
            return Err(eyre::eyre!("no outputs given"));
        }

        for name_or_id in args {
            let id = self.output_for(&name_or_id)?;

            self.client.out_toggle(id)?;
        }

        self.outputs()
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

    pub fn load(
        &mut self,
        name: &String,
        range: Option<String>,
    ) -> eyre::Result<Option<String>> {
        match range {
            Some(range_str) => {
                let parts: Vec<u32> = range_str
                    .split(':')
                    .filter_map(|s| s.parse().ok())
                    .collect();

                match parts.as_slice() {
                    [start, end] if start < end && *start > 0 && *end > 0 => {
                        self.client.load(name, *start..*end)?;
                    }
                    [start, end] if start >= end => {
                        return Err(eyre::eyre!(
                            "end cannot be less than or equal to start"
                        ));
                    }
                    _ => {
                        return Err(eyre::eyre!(
                        "invalid range, should be 'start:end' where start and end > 0."
                    ));
                    }
                }
            }
            None => {
                self.client.load(name, ..)?;
            }
        }

        Ok(Some(format!("loading: {name}")))
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

    pub fn listall(
        &mut self,
        file: Option<&str>,
    ) -> eyre::Result<Option<String>> {
        let all_files = Listing::from(self.client.listall()?);

        let songs = if let Some(ref file) = file {
            // TODO: this is inefficient but it's the only way I see at the moment
            all_files
                .listing
                .iter()
                .filter(|song| song.starts_with(file))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            all_files.listing.clone()
        };

        let files = Listing::from(songs);

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&files)?,
            OutputFormat::Text => files.to_string(),
        };

        Ok(Some(response))
    }

    pub fn ls(
        &mut self,
        directory: Option<&str>,
    ) -> eyre::Result<Option<String>> {
        let directory = directory.unwrap_or("");
        let listing = self.client.listfiles(directory)?;
        let filter_for = if let Some(entry) = listing.first() {
            entry.0.as_str()
        } else {
            "directory"
        };

        let results = Listing::from(
            listing
                .clone()
                .into_iter()
                .filter(|(key, _)| key == filter_for)
                .map(|(_, value)| {
                    PathBuf::from(&directory)
                        .join(value)
                        .to_str()
                        .unwrap()
                        .to_string()
                })
                .collect::<Vec<String>>(),
        );

        let response = match self.format {
            OutputFormat::Json => serde_json::to_string(&results)?,
            OutputFormat::Text => results.to_string(),
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
        let time = crate::time::Track::from(status.time);

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
