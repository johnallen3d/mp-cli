#![deny(clippy::pedantic)]
extern crate chrono;
extern crate mpd;
extern crate serde_json;

use clap::Parser;

mod args;
mod client;

use args::{Cli, Commands};

fn main() {
    let args = Cli::parse();

    // safe to unwrap because we have default values
    let mut mpd = match crate::client::Client::new(
        &args.bind_to_address.unwrap(),
        &args.port.unwrap(),
        args.format.clone(),
    ) {
        Ok(client) => client,
        Err(e) => handle_error(e),
    };

    let result = match args.command {
        Some(Commands::Crop) => mpd.crop(),
        Some(Commands::Del { position }) => mpd.del(position),
        Some(Commands::Current) => mpd.current(),
        Some(Commands::Play { position }) => mpd.play(position),
        Some(Commands::Next) => mpd.next(),
        Some(Commands::Prev) => mpd.prev(),
        Some(Commands::Pause) => mpd.pause(),
        Some(Commands::PauseIfPlaying) => mpd.pause_if_playing(),
        Some(Commands::Toggle) => mpd.toggle(),
        Some(Commands::Cdprev) => mpd.cdprev(),
        Some(Commands::Stop) => mpd.stop(),

        Some(Commands::Clear) => mpd.clear(),
        Some(Commands::Outputs) => mpd.outputs(),
        Some(Commands::Queued) => mpd.queued(),
        Some(Commands::Shuffle) => mpd.shuffle(),
        Some(Commands::Lsplaylists) => mpd.lsplaylists(),
        Some(Commands::Repeat { state }) => mpd.repeat(state),
        Some(Commands::Random { state }) => mpd.random(state),
        Some(Commands::Single { state }) => mpd.single(state),
        Some(Commands::Consume { state }) => mpd.consume(state),

        Some(Commands::Volume { volume }) => mpd.set_volume(&volume),
        Some(Commands::Stats) => mpd.stats(),
        Some(Commands::Version) => mpd.version(),

        Some(Commands::Status) | None => mpd.current_status(),
    };

    match result {
        Ok(Some(output)) => println!("{output}"),
        Ok(None) => (),
        Err(e) => handle_error(e),
    }
}

fn handle_error(error: impl std::fmt::Display) -> ! {
    let err_text = error.to_string();
    if !err_text.is_empty() {
        println!("{err_text}");
    }
    std::process::exit(1);
}
