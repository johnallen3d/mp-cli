#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
use std::fmt;

extern crate chrono;
extern crate mpd;
extern crate serde_json;

use serde::Serialize;

mod client;
mod range;
mod se;
mod song;
mod stats;
mod status;
mod time;

pub use client::Client;
pub use status::State;

pub enum OutputFormat {
    Text,
    Json,
    None,
}

#[derive(Debug, PartialEq, Serialize)]
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
