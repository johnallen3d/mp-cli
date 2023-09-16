# mpc-rs

A Music Player Daemon (MPD) CLI client implemented in Rust.

## Features

- Playback control: `play`/`pause`, `toggle` etc.
- Volume adjustment: `volume 50`, `volume +10`, `volume -- -10` ([issue](https://github.com/johnallen3d/mpc-rs/issues/1))
- Status: `status`

### Full Help

```bash
❯ mpc-rs help
Music Player Daemon client written in Rust

Usage: mpc-rs [OPTIONS] [COMMAND]

Commands:
  play    Start the player
  next    Next song in the queue
  prev    Previous song in the queue
  pause   Pause the player
  toggle  Toggle play/pause
  stop    Stop the player
  clear   Clear the current playlist
  queued  Display the next song in the queue
  volume  Set the volume to specified value <num> or increase/decrease it [+-]<num>
  status  Get the current status of the player
  help    Print this message or the help of the given subcommand(s)

Options:
      --format <FORMAT>  Set output format [default: txt] [possible values: txt, json]
  -h, --help             Print help
```

## Installation

At this time it is necessary to compile and install the crate locally. The simplest way to do this is to [install the Rust toolchain](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then use `cargo` to build and install locally.

```bash
git clone https://github.com/johnallen3d/mpc-rs.git
cd mpc-rs
cargo install --path .
```

## Why?

Mostly because I wanted to practice writing Rust. Also, for use with the wonderful macOS bar app [SketchyBar](https://github.com/FelixKratz/SketchyBar). One of the plugins I've created displays the current status (playing/paused) of MPD along with the artist and title. I've used [mpc](https://github.com/MusicPlayerDaemon/mpc) for this purpose in the past but the status output is not well suited for parsing (more for [human readability](https://github.com/MusicPlayerDaemon/mpc/issues/65#issuecomment-982840758)).

With this in mind I decided to implement a client that would provide a more consistent and parseable output.

```bash
❯ mpc-rs status
volume=100
state=play
artist=King Gizzard & The Lizard Wizard
title=Wah Wah
```

This format is easily and efficiently parseable in a shell script:

```bash
#! /usr/bin/env bash

status=$(mpc-rs)

while IFS='=' read -r key value; do
	case "$key" in
	'artist') artist="$value" ;;
	'state') state="$value" ;;
	'title') title="$value" ;;
	'volume') volume="$value" ;;
	esac
done <<<"$status"

echo "${artist}"
echo "${state}"
echo "${title}"
echo "${volume}"
```

Alternatively the status can be presented as JSON.

```bash
❯ mpc-rs --format json status | jq
{
  "volume": "100",
  "state": "play",
  "artist": "King Gizzard & The Lizard Wizard",
  "title": "Road Train"
}
```

## Credit

All of the heavy lifting of communicating with the daemon is handled by [rust-mpd](https://crates.io/crates/mpd).

Obviously [mpc](https://github.com/MusicPlayerDaemon/mpc) the real CLI tool for interacting with `mpd`. For the commands that I have (currently) implemented I've done my best to mirror the interface of `mpc`. In theory this could be a drop in replacement, for the very limited use-case.
