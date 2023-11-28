use std::collections::HashMap;
use std::env;

use crossterm::style::Stylize;

use crate::data::{DATA_DIR, GOLD, RANK};

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
    env::args().any(|arg| arg == "--practice")
}

pub(crate) fn must_run_solutions() -> bool {
    env::args().any(|arg| arg == "--force-run")
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
    .to_std()
    .expect("Should never be negative")
    .as_secs_f64()
}

pub(crate) fn format_time(seconds: f64) -> String {
    let (minutes, seconds) = ((seconds / 60.0).trunc(), seconds % 60.0);
    let (hours, minutes) = ((minutes / 60.0).trunc(), minutes % 60.0);
    if hours > 0.0 {
        format!("{hours:02.0}:{minutes:02.0}:{:02.0}", seconds.trunc())
    } else {
        format!("{minutes:02.0}:{seconds:05.2}")
    }
}

#[cfg(feature = "web")]
pub(crate) fn message_from_body(body: &str) -> String {
    use tl::ParserOptions;

    let page =
        tl::parse(body, ParserOptions::new()).expect("Failed to parse response.");
    let article = page
        .query_selector("article")
        .expect("Failed to compile the 'article' query")
        .next()
        .expect("`article` tag missing from response")
        .get(page.parser())
        .expect("Failed to retrieve node associated with the `article` tag");
    article.inner_text(page.parser()).to_string()
}

pub(crate) fn print_rank(msg: &str) {
    if let Some(rank) = RANK.captures(msg) {
        pretty_print(&format!(
            "You got rank {} for this puzzle",
            rank.get(1).expect("RANK regex has one capture").as_str()
        ));
    }
}

#[cfg(feature = "web")]
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Submissions {
    #[serde(rename = "1")]
    pub part_1: HashMap<String, String>,
    #[serde(rename = "2")]
    pub part_2: HashMap<String, String>,
}

#[cfg(feature = "web")]
mod test_info {
    #![allow(clippy::option_option)]
    pub fn double_option<
        'de,
        T: serde::Deserialize<'de>,
        D: serde::Deserializer<'de>,
    >(
        de: D,
    ) -> Result<Option<Option<T>>, D::Error> {
        serde::Deserialize::deserialize(de).map(Some)
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TestInfo {
        #[serde(rename = "1", deserialize_with = "double_option")]
        pub part_1: Option<Option<(String, String)>>,
        #[serde(rename = "2", deserialize_with = "double_option")]
        pub part_2: Option<Option<(String, String)>>,
    }
}
#[cfg(feature = "web")]
pub(crate) use test_info::TestInfo;
