# mp-cli
[![ci](https://github.com/johnallen3d/mp-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/johnallen3d/mp-cli/actions/workflows/ci.yml) [![Coverage Status](https://coveralls.io/repos/github/johnallen3d/mpc-rs/badge.svg?branch=main)](https://coveralls.io/github/johnallen3d/mpc-rs?branch=main) [![Crate Status](https://img.shields.io/crates/v/mp-cli.svg)](https://crates.io/crates/mp-cli)

A Music Player Daemon (MPD) CLI client implemented in Rust.

## Features

I'm working towards compatibility with [mpc](https://github.com/MusicPlayerDaemon/mpc) over time. See [this tracking issue](https://github.com/johnallen3d/mp-cli/issues/16) for the current status.

Alternatively, see the [help](./doc/help.txt).

## Installation

At this time it is necessary to compile and install the crate locally. The simplest way to do this is to [install the Rust toolchain](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then use `cargo` to build and install from [crates.io](https://crates.io/crates/mp-cli).

```bash
cargo install mp-cli
```

## Why?

Mostly because I wanted to practice writing Rust. Also, for use with the wonderful macOS bar app [SketchyBar](https://github.com/FelixKratz/SketchyBar). One of the plugins I've created displays the current status (playing/paused) of MPD along with the artist and title. I've used [mpc](https://github.com/MusicPlayerDaemon/mpc) for this purpose in the past but the status output is not well suited for parsing (more for [human readability](https://github.com/MusicPlayerDaemon/mpc/issues/65#issuecomment-982840758)).

With this in mind I decided to implement a client that would provide a more consistent and parseable output.

```bash
❯ mp-cli status
volume=100
state=play
artist=King Gizzard & The Lizard Wizard
title=Wah Wah
```

This format is easily and efficiently parseable in a shell script:

```bash
#! /usr/bin/env bash

status=$(mp-cli)

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
❯ mp-cli --format json status | jq
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
