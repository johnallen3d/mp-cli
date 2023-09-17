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
    ) {
        Ok(client) => client,
        Err(e) => handle_error(e),
    };

    let result = match args.command {
        Some(Commands::Current) => mpd.current(),
        Some(Commands::Play) => mpd.play(),
        Some(Commands::Next) => mpd.next(),
        Some(Commands::Prev) => mpd.prev(),
        Some(Commands::Pause) => mpd.pause(),
        Some(Commands::PauseIfPlaying) => mpd.pause_if_playing(),
        Some(Commands::Toggle) => mpd.toggle(),
        Some(Commands::Cdprev) => mpd.cdprev(),
        Some(Commands::Stop) => mpd.stop(),

        Some(Commands::Clear) => mpd.clear(),
        Some(Commands::Queued) => mpd.queued(),
        Some(Commands::Shuffle) => mpd.shuffle(),
        Some(Commands::Repeat { state }) => mpd.repeat(state, args.format),
        Some(Commands::Random { state }) => mpd.random(state, args.format),
        Some(Commands::Single { state }) => mpd.single(state, args.format),

        Some(Commands::Volume { volume }) => mpd.set_volume(&volume),

        Some(Commands::Status) => mpd.current_status(args.format),
        None => Ok(None),
    };

    match result {
        Ok(Some(output)) => println!("{}", output),
        Ok(None) => (),
        Err(e) => handle_error(e),
    }
}

fn handle_error(error: impl std::fmt::Display) -> ! {
    let err_text = error.to_string();
    if !err_text.is_empty() {
        println!("{}", err_text);
    }
    std::process::exit(1);
}
