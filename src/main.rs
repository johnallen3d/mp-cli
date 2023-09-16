extern crate mpd;
extern crate serde_json;

use clap::Parser;

mod args;
mod client;

use args::{Cli, Commands};

fn main() {
    let args = Cli::parse();

    let mut mpd = match crate::client::Client::new() {
        Ok(client) => client,
        Err(e) => handle_error(e),
    };

    let result = match args.command {
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

        Some(Commands::Volume { volume }) => mpd.set_volume(&volume),

        Some(Commands::Status) => Ok(()), // always show status (below)
        None => Ok(()),                   // default to showing status (below)
    };

    if let Err(e) = result.and_then(|_| {
        mpd.current_status().map(|status| match args.format {
            args::OutputFormat::Json => {
                println!("{}", serde_json::to_string(&status).unwrap())
            }
            args::OutputFormat::Text => println!("{}", status),
        })
    }) {
        handle_error(e);
    }
}

fn handle_error(error: impl std::fmt::Display) -> ! {
    let err_text = error.to_string();
    if !err_text.is_empty() {
        println!("{}", err_text);
    }
    std::process::exit(1);
}
