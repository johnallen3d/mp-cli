#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
use std::io::BufRead;

use clap::Parser;

mod args;

use args::{Cli, Commands, OnOff};
use mpd_easy::Client;

fn main() {
    let args = Cli::parse();

    // safe to unwrap because we have default values
    let mut mpd = match Client::new(
        &args.bind_to_address.unwrap(),
        &args.port.unwrap(),
        args.format.to(),
    ) {
        Ok(client) => client,
        Err(e) => handle_error(e),
    };

    let result = match args.command {
        Some(Commands::Add { path }) => {
            mpd.add(&input_or_stdin(path, std::io::stdin().lock()))
        }
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
        Some(Commands::Seek { position }) => mpd.seek(&position),
        Some(Commands::Seekthrough { position }) => mpd.seekthrough(&position),

        Some(Commands::Clear) => mpd.clear(),
        Some(Commands::Outputs) => mpd.outputs(),
        Some(Commands::Enable { args }) => mpd.enable(args),
        Some(Commands::Disable { args }) => mpd.disable(args),
        Some(Commands::Toggleoutput { args }) => mpd.toggle_output(args),
        Some(Commands::Queued) => mpd.queued(),
        Some(Commands::Shuffle) => mpd.shuffle(),
        Some(Commands::Lsplaylists) => mpd.lsplaylists(),
        Some(Commands::Load { name, range }) => mpd.load(&name, range),
        Some(Commands::Insert { uri }) => {
            mpd.insert(&input_or_stdin(uri, std::io::stdin().lock()))
        }
        Some(Commands::Prio {
            priority,
            position_or_range,
        }) => mpd.prio(&priority, &position_or_range),
        Some(Commands::Playlist { name }) => mpd.playlist(name),
        Some(Commands::Listall { file }) => mpd.listall(file.as_deref()),
        Some(Commands::Ls { directory }) => mpd.ls(directory.as_deref()),
        Some(Commands::Repeat { state }) => mpd.repeat(OnOff::to(&state)),
        Some(Commands::Random { state }) => mpd.random(OnOff::to(&state)),
        Some(Commands::Single { state }) => mpd.single(OnOff::to(&state)),
        Some(Commands::Search { tag, query }) => {
            mpd.search(tag.to_str(), &query)
        }
        Some(Commands::SearchAdd { tag, query }) => {
            mpd.search_add(tag.to_str(), &query)
        }
        Some(Commands::Find { tag, query }) => mpd.find(tag.to_str(), &query),
        Some(Commands::FindAdd { tag, query }) => {
            mpd.find_add(tag.to_str(), &query)
        }
        Some(Commands::List { tag }) => mpd.list(tag.to_str()),
        Some(Commands::Consume { state }) => mpd.consume(OnOff::to(&state)),
        Some(Commands::Crossfade { seconds }) => mpd.crossfade(seconds),

        Some(Commands::Save { name }) => mpd.save(&name),
        Some(Commands::Rm { name }) => mpd.rm(&name),
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

fn input_or_stdin<R: BufRead>(path: Option<String>, reader: R) -> String {
    if let Some(p) = path {
        return p;
    }

    let mut buffer = String::new();
    let mut reader = reader;

    reader
        .read_line(&mut buffer)
        .expect("error reading from input");

    buffer.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_input_or_stdin_with_path() {
        let path = Some("some_path".to_string());
        let cursor = Cursor::new("not_used");

        let result = input_or_stdin(path, cursor);
        assert_eq!(result, "some_path");
    }

    #[test]
    fn test_input_or_stdin_with_stdin() {
        let cursor = Cursor::new("from_stdin\n");

        let result = input_or_stdin(None, cursor);
        assert_eq!(result, "from_stdin");
    }
}
