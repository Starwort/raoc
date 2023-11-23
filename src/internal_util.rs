use std::fmt::Display;
use std::path::Path;
use std::sync::atomic::{self, AtomicBool};
use std::time::{Duration, Instant};
use std::{fs, io, thread};

use crossterm::style::{style, Stylize};
use lazy_static::lazy_static;

use crate::data::{DATA_DIR, GOLD, TOKEN_FILE, USER_AGENT};

pub(crate) fn strip_trailing_nl(mut input: String) -> String {
    let new_len = input
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, '\n' | '\r'))
        .map_or(0, |(i, _)| i + 1);
    if new_len != input.len() {
        input.truncate(new_len);
    }
    input
}

#[cfg(feature = "async")]
#[cfg(all(not(feature = "sync"), feature = "async"))]
pub use async_wait as wait;
#[cfg(all(not(feature = "sync"), feature = "async"))]
pub use async_work as work;

/// Open the page, if the user hasn't opted out.
pub(crate) fn open_page(url: &str) {
    if !(&*DATA_DIR / ".nobrowser").exists() {
        webbrowser::open(url).expect("Failed to open web browser.");
    }
}

/// Analyse and print message
pub(crate) fn pretty_print(message: &str) {
    if message.starts_with("That's the") {
        println!("{}", message.green());
    } else if message.starts_with("You don't") {
        println!("{}", message.yellow());
    } else if message.starts_with("That's not") {
        println!("{}", message.red());
    } else if message.starts_with("You got rank") {
        println!("{}", message.on(GOLD));
    } else {
        eprintln!("WARN: Couldn't parse message");
        println!("{message}");
    }
}

pub(crate) fn is_practice_mode() -> bool {
    std::env::args().any(|arg| arg == "--practice")
}

#[cfg(any(feature = "sync", feature = "async"))]
#[allow(deprecated, clippy::cast_precision_loss)]
pub(crate) fn get_leaderboard_time(day: u32, time: &str) -> f64 {
    (chrono::NaiveDateTime::parse_from_str(
        &format!("{time} 1900"),
        "%b %d  %H:%M:%S %Y",
    )
    .expect("Failed to parse time")
        - chrono::NaiveDate::from_ymd(1900, 12, day).and_hms(0, 0, 0))
    .num_seconds() as f64
}
