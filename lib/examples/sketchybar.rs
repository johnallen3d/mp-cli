#!/usr/bin/env -S cargo +nightly --quiet -Zscript -Zgc
```cargo
[dependencies]
mpd-easy = "0.1.5"
sketchybar-rs = "0.2.0"
```

fn main() {
    let format = mpd_easy::OutputFormat::Json;
    let mut client =
        mpd_easy::Client::new("localhost", "6600", format).unwrap();

    let status = client.status().unwrap();

    let mut label = String::new();

    let icon = match status.state {
        mpd_easy::State::Play => "".to_string(),
        mpd_easy::State::Pause => "".to_string(),
        mpd_easy::State::Stop => "".to_string(),
    };

    if status.state != mpd_easy::State::Stop {
        label = format!(
            "{} • {} • {} [{}/{}]",
            status.title,
            status.artist,
            status.album,
            status.elapsed,
            status.track_length,
        );
    }

    let message = format!("--set mpd icon=\"{icon}\" label=\"{label}\"");

    sketchybar_rs::message(&message, Some("bottombar")).unwrap();
}
