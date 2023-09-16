use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Music Player Daemon client written in Rust
#[derive(Debug, Parser)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
    /// Set output format
    #[clap(long, value_enum, default_value_t=OutputFormat::Json)]
    pub(crate) format: OutputFormat,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    //
    // playback related commands
    //
    /// Print the current song
    #[command()]
    Current,
    /// Start the player
    #[command()]
    Play,
    /// Next song in the queue
    #[command()]
    Next,
    /// Previous song in the queue
    #[command()]
    Prev,
    /// Pause the player
    #[command()]
    Pause,
    /// Pause the player if it's playing
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
    /// Display the next song in the queue
    #[command()]
    Queued,

    //
    // volume related commands
    //
    /// Set the volume to specified value <num> or increase/decrease it [+-]<num>
    #[command()]
    Volume { volume: String },

    //
    // output related commands
    //
    /// Get the current status of the player
    #[command()]
    Status,
}
