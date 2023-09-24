use std::fmt;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

/// Music Player Daemon client written in Rust
#[derive(Debug, Parser)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
    /// Set output format
    #[clap(long, value_enum, default_value_t=OutputFormat::Json)]
    pub(crate) format: OutputFormat,
    /// Set the ip address the mpd server is listening on
    #[clap(long, default_value = "127.0.0.1")]
    pub(crate) bind_to_address: Option<String>,
    /// Set the port the mpd server is listening on
    #[clap(long, default_value = "6600")]
    pub(crate) port: Option<String>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    //
    // queue related commands
    //
    /// Add a song to the queue at the given path (or dir '/')
    #[command()]
    Add { path: Option<String> },
    /// Remove all but the current song from the queue
    #[command()]
    Crop,
    /// Print the current song
    #[command()]
    Current,
    /// Print the current song
    #[command()]
    Del { position: Option<u32> },

    //
    // playback related commands
    //
    /// Start the player
    #[command()]
    Play { position: Option<u32> },
    /// Next song in the queue
    #[command()]
    Next,
    /// Previous song in the queue
    #[command()]
    Prev,
    /// Pause the player
    #[command()]
    Pause,
    /// Pause the player if it is playing
    #[command()]
    PauseIfPlaying,
    /// CD player like previous song
    #[command()]
    Cdprev,
    /// Toggle play/pause
    #[command()]
    Toggle,
    /// Stop the player
    #[command()]
    Stop,

    //
    // playlist related commands
    //
    /// Clear the current playlist
    #[command()]
    Clear,
    /// List current outputs
    #[command()]
    Outputs,
    /// Display the next song in the queue
    #[command()]
    Queued,
    /// Shuffle the queue
    #[command()]
    Shuffle,
    /// List all of the playlists
    #[command()]
    Lsplaylists,
    /// List songs in a playlist
    #[command()]
    Playlist { name: Option<String> },
    /// Toggle repeat mode or set to provided state
    #[command()]
    Repeat { state: Option<OnOff> },
    /// Toggle random mode or set to provided state
    #[command()]
    Random { state: Option<OnOff> },
    /// Toggle single mode or set to provided state
    #[command()]
    Single { state: Option<OnOff> },
    /// Toggle consume mode or set to provided state
    #[command()]
    Consume { state: Option<OnOff> },
    /// Toggle consume mode or set to provided state
    #[command()]
    Crossfade { seconds: Option<String> },

    /// Save queue to a playlist
    #[command()]
    Save { name: String },
    /// Remove a playlist
    #[command()]
    Rm { name: String },
    /// Set the volume to specified value <num> or increase/decrease it [+-]<num>
    #[command()]
    Volume { volume: String },

    /// Provide mpd statistics
    #[command()]
    Stats,
    /// Provide the mpd version and the mp-cli version
    #[command()]
    Version,

    //
    // output related commands
    //
    /// Get the current status of the player
    #[command()]
    Status,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, PartialEq, Serialize, ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

impl From<bool> for OnOff {
    fn from(value: bool) -> Self {
        if value {
            OnOff::On
        } else {
            OnOff::Off
        }
    }
}

impl fmt::Display for OnOff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OnOff::On => write!(f, "on"),
            OnOff::Off => write!(f, "off"),
        }
    }
}
