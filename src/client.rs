use eyre::WrapErr;
use serde::Serialize;
// use serde_json::to_string;
use std::fmt;

#[derive(Serialize)]
pub struct Status {
    volume: String,
    state: String,
    artist: String,
    title: String,
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
    pub fn new() -> eyre::Result<Client> {
        // TODO: read connection details from mpd.conf
        let client = mpd::Client::connect("127.0.0.1:6600")
            .wrap_err("Error connecting to mpd server".to_string())?;

        Ok(Self { client })
    }

    //
    // playback related commands
    //

    pub fn play(&mut self) -> eyre::Result<()> {
        self.client.play().map_err(|e| eyre::eyre!(e))
    }

    pub(crate) fn next(&mut self) -> Result<(), eyre::Error> {
        self.client.next().map_err(|e| eyre::eyre!(e))
    }

    pub(crate) fn prev(&mut self) -> Result<(), eyre::Error> {
        self.client.prev().map_err(|e| eyre::eyre!(e))
    }

    pub fn pause(&mut self) -> eyre::Result<()> {
        self.client.pause(true).map_err(|e| eyre::eyre!(e))
    }

    pub fn toggle(&mut self) -> eyre::Result<()> {
        match self.client.status()?.state {
            mpd::State::Play => self.pause()?,
            mpd::State::Pause => self.play()?,
            mpd::State::Stop => self.play()?,
        };

        Ok(())
    }

    pub fn stop(&mut self) -> eyre::Result<()> {
        self.client.stop().map_err(|e| eyre::eyre!(e))
    }

    //
    // playlist related commands
    //

    pub(crate) fn clear(&mut self) -> Result<(), eyre::Error> {
        self.client.clear().map_err(|e| eyre::eyre!(e))
    }

    pub(crate) fn queued(&mut self) -> Result<(), eyre::Error> {
        if let Some(song) =
            self.client.queue().map_err(|e| eyre::eyre!(e))?.get(0)
        {
            println!("{}", song.title.as_ref().unwrap_or(&"".to_string()));
        }

        Ok(())
    }

    //
    // volume related commands
    //

    pub fn set_volume(&mut self, input: &str) -> eyre::Result<()> {
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

        self.client.volume(target).map_err(|e| eyre::eyre!(e))
    }

    //
    // output related commands
    //

    pub fn current_status(&mut self) -> eyre::Result<Status> {
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

        let status = Status {
            volume,
            state,
            artist,
            title,
        };

        // Ok(to_string(&status)?)
        Ok(status)
    }
}
