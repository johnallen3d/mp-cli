# Music Player Daemon (MPD) Easy

An MPD client library that wraps the [mpd](https://crates.io/crates/mpd) crate providing an interface that closely resembles the [mpc](https://www.musicpd.org/doc/mpc/html/) commands.

## Why?

This was created initially for fun and practice writing Rust. The library code here was written in support of [`mp-cli`](https://github.com/johnallen3d/mp-cli).

### SketcyBar

Recently I've been playing around with the Rust nightly feature [`cargo-script`](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#script) in conjunction with [SketchBar](https://github.com/FelixKratz/SketchyBar). I wanted to see if I could write a SketchyBar plugin that displays current MPD status entirely in Rust. Combining this library, my [Rust SketchyBar helper](https://github.com/johnallen3d/sketchybar-rs) and `cargo-script` I was able to do just that.

![sketchybar-mpd-example](./examples/sketchybar-mpd.png)

See the [sketchybar](./examples/sketchybar.rs) example for more details.
