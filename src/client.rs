use std::time::Duration;

use eyre::WrapErr;
use serde::Serialize;
use std::fmt;

use crate::args::{OnOff, OutputFormat};

#[derive(Serialize)]
pub struct Status {
    volume: String,
    state: String,
    artist: String,
    title: String,
    repeat: OnOff,
    random: OnOff,
    single: OnOff,
    consume: OnOff,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "volume={}\nstate={}\nartist={}\ntitle={}",
            self.volume, self.state, self.artist, self.title
        )
    }
}

pub struct Client {
    client: mpd::Client,
}

impl Client {
    pub fn new(bind_to_address: &str, port: &str) -> eyre::Result<Client> {
        // TODO: read connection details from mpd.conf
        let client = mpd::Client::connect(format!("{bind_to_address}:{port}"))
            .wrap_err("Error connecting to mpd server".to_string())?;

        Ok(Self { client })
    }

    //
    // playback related commands
    //
    pub fn current(&mut self) -> eyre::Result<Option<String>> {
        let status = self.status()?;

        Ok(Some(format!("{} - {}", status.artist, status.title)))
    }

    pub fn play(&mut self) -> eyre::Result<Option<String>> {
        self.client.play().map(|_| None).map_err(eyre::Report::from)
    }

    pub fn next(&mut self) -> eyre::Result<Option<String>> {
        self.client.next().map(|_| None).map_err(eyre::Report::from)
    }

    pub fn prev(&mut self) -> eyre::Result<Option<String>> {
        self.client.prev().map(|_| None).map_err(eyre::Report::from)
    }

    pub fn pause(&mut self) -> eyre::Result<Option<String>> {
        self.client
            .pause(true)
            .map(|_| None)
            .map_err(eyre::Report::from)
    }

    pub fn pause_if_playing(&mut self) -> eyre::Result<Option<String>> {
        match self.client.status()?.state {
            mpd::State::Play => {
                self.pause()?;
                Ok(None)
            }
            mpd::State::Pause => Err(eyre::eyre!("")),
            mpd::State::Stop => Err(eyre::eyre!("")),
        }
    }

    pub fn cdprev(&mut self) -> eyre::Result<Option<String>> {
        let default_duration = Duration::from_secs(0);
        let status = &self.client.status()?;
        let current = status.elapsed.unwrap_or(default_duration).as_secs();

        if current < 3 {
            self.prev()?;
        } else {
            let place = match status.song {
                Some(ref song) => song.pos,
                None => 0,
            };
            self.client.seek(place, 0)?;
        }

        Ok(None)
    }

    pub fn toggle(&mut self) -> eyre::Result<Option<String>> {
        match self.client.status()?.state {
            mpd::State::Play => self.pause()?,
            mpd::State::Pause => self.play()?,
            mpd::State::Stop => self.play()?,
        };

        Ok(None)
    }

    pub fn stop(&mut self) -> eyre::Result<Option<String>> {
        self.client.stop().map(|_| None).map_err(eyre::Report::from)
    }

    //
    // playlist related commands
    //

    pub fn clear(&mut self) -> eyre::Result<Option<String>> {
        self.client
            .clear()
            .map(|_| None)
            .map_err(eyre::Report::from)
    }

    pub fn queued(&mut self) -> eyre::Result<Option<String>> {
        if let Some(song) =
            self.client.queue().map_err(|e| eyre::eyre!(e))?.get(0)
        {
            Ok(Some(
                song.title.as_ref().unwrap_or(&"".to_string()).to_owned(),
            ))
        } else {
            Ok(None)
        }
    }

    pub fn shuffle(&mut self) -> eyre::Result<Option<String>> {
        self.client.shuffle(..)?;

        Ok(None)
    }

    pub fn repeat(
        &mut self,
        state: Option<OnOff>,
        format: OutputFormat,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.repeat,
        };

        self.client.repeat(state)?;

        self.current_status(format)
    }

    pub(crate) fn random(
        &mut self,
        state: Option<OnOff>,
        format: OutputFormat,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.random,
        };

        self.client.random(state)?;

        self.current_status(format)
    }

    pub(crate) fn single(
        &mut self,
        state: Option<OnOff>,
        format: OutputFormat,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.single,
        };

        self.client.single(state)?;

        self.current_status(format)
    }

    pub fn consume(
        &mut self,
        state: Option<OnOff>,
        format: OutputFormat,
    ) -> eyre::Result<Option<String>> {
        let state = match state {
            Some(state) => state == OnOff::On,
            None => !self.client.status()?.consume,
        };

        self.client.consume(state)?;

        self.current_status(format)
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

        Ok(Status {
            volume,
            state,
            artist,
            title,
            repeat: OnOff::from(status.repeat),
            random: OnOff::from(status.random),
            single: OnOff::from(status.single),
            consume: OnOff::from(status.consume),
        })
    }

    pub fn current_status(
        &mut self,
        format: OutputFormat,
    ) -> eyre::Result<Option<String>> {
        let status = self.status()?;
        let response = match format {
            OutputFormat::Json => serde_json::to_string(&status)?,
            OutputFormat::Text => format!("{}", status),
        };

        Ok(Some(response))
    }
}
