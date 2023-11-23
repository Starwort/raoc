use std::fmt::Display;
use std::path::Path;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};
use std::{fs, io, thread};

use crossterm::style::{style, Stylize};
use lazy_static::lazy_static;
use reqwest::blocking::{Client, Response};
use reqwest::header;

use crate::data::{get_cookie, DATA_DIR, TOKEN_FILE, USER_AGENT};
use crate::internal_util::{get_leaderboard_time, strip_trailing_nl};

pub(crate) fn load_token_from_stdin(why: impl Display) -> String {
    eprintln!("{why}");
    eprint!(">>> ");
    let mut token = String::new();
    io::stdin()
        .read_line(&mut token)
        .expect("Failed to read token.");
    token = strip_trailing_nl(token);
    fs::write(&*TOKEN_FILE, &token)
        .expect("Failed to write token file. Check your permissions.");
    token
}

/// Wait the specified time, displaying a countdown, a spinner, and a message.
pub fn wait(msg: impl Display, time: Duration) {
    let start = Instant::now();
    let end = start + time;
    let mut time_left = end - start;
    while {
        eprint!(
            "\r{} {} {:02}{}{:02}{}{:02}",
            msg,
            match (start.elapsed().as_millis() / 80) % 10 {
                0 => '⠋',
                1 => '⠙',
                2 => '⠹',
                3 => '⠸',
                4 => '⠼',
                5 => '⠴',
                6 => '⠦',
                7 => '⠧',
                8 => '⠇',
                9 => '⠏',
                _ => unreachable!(),
            }
            .yellow(),
            style(time_left.as_secs() / 3600).yellow(),
            ':'.yellow(),
            style(time_left.as_secs() / 60 % 60).yellow(),
            ':'.yellow(),
            style(time_left.as_secs() % 60).yellow(),
        );
        time_left.as_secs() > 0
    } {
        thread::sleep(Duration::from_millis(100));
        time_left = end - Instant::now();
    }
    eprintln!();
}

/// Run the given worker function, displaying a message, spinner, and elapsed
/// timer.
pub fn work<T, U>(msg: impl Display + Sync, worker: impl FnOnce(T) -> U, data: T) -> U {
    let start = Instant::now();
    let is_done = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            while !is_done.load(atomic::Ordering::Relaxed) {
                let elapsed = start.elapsed();
                eprint!(
                    "\r{} {} {:02}{}{:02}{}{:02}",
                    msg,
                    match (start.elapsed().as_millis() / 80) % 10 {
                        0 => '⠋',
                        1 => '⠙',
                        2 => '⠹',
                        3 => '⠸',
                        4 => '⠼',
                        5 => '⠴',
                        6 => '⠦',
                        7 => '⠧',
                        8 => '⠇',
                        9 => '⠏',
                        _ => unreachable!(),
                    }
                    .yellow(),
                    style(elapsed.as_secs() / 3600).yellow(),
                    ':'.yellow(),
                    style(elapsed.as_secs() / 60 % 60).yellow(),
                    ':'.yellow(),
                    style(elapsed.as_secs() % 60).yellow(),
                );
                thread::sleep(Duration::from_millis(100));
            }
            eprintln!();
        });
        let result = worker(data);
        // let the spinner thread die in its own time
        is_done.store(true, atomic::Ordering::Relaxed);
        result
    })
}

/// Make a directory, if it doesn't exist.
pub(crate) fn make(dir: &Path) {
    if !dir.exists() {
        fs::create_dir_all(dir).expect("Failed to create data directory.");
    }
}

lazy_static! {
    static ref CLIENT: Client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to build reqwest client.");
}

pub(crate) fn get(url: &str, authenticate: bool) -> Response {
    if authenticate {
        CLIENT.get(url).header(header::COOKIE, get_cookie())
    } else {
        CLIENT.get(url)
    }
    .send()
    .expect("Advent of Code sent back a bad response, or the network is down.")
}

pub(crate) fn get_text(url: &str, authenticate: bool) -> String {
    get(url, authenticate)
        .text()
        .expect("Advent of Code sent back a bad response")
}

pub(crate) fn post(url: &str, authenticate: bool) -> reqwest::blocking::Response {
    if authenticate {
        CLIENT.post(url).header(header::COOKIE, get_cookie())
    } else {
        CLIENT.post(url)
    }
    .send()
    .expect("Advent of Code sent back a bad response, or the network is down.")
}

pub(crate) fn post_text(url: &str, authenticate: bool) -> String {
    post(url, authenticate)
        .text()
        .expect("Advent of Code sent back a bad response")
}

pub(crate) fn load_leaderboard_times(day: u32, year: i32) -> (Vec<f64>, Vec<f64>) {
    use crate::data::leaderboard_url;

    let day_dir = &*DATA_DIR / year.to_string() / day.to_string();
    make(&day_dir);

    let leaderboards = day_dir / "leaderboards.json";
    if leaderboards.exists() {
        let data = fs::read_to_string(&leaderboards)
            .expect("Failed to read leaderboards file.");
        serde_json::from_str(&data).expect("Failed to parse leaderboard cache.")
    } else {
        let leaderboard_page = get_text(&leaderboard_url(year, day), false);
        let soup = tl::parse(&leaderboard_page, tl::ParserOptions::new())
            .expect("Parsing the leaderboard page failed.");
        let times = soup
            .query_selector(".leaderboard-entry")
            .expect("Selector is always valid");
        let mut part_1_times = Vec::new();
        let mut part_2_times = Vec::new();
        let mut in_part_2 = false;
        for time in times {
            let time = time
                .get(soup.parser())
                .expect("`time` will always be from `soup`")
                .as_tag()
                .expect("Node is always a tag");
            let position = time
                .query_selector(soup.parser(), ".leaderboard-position")
                .expect("Selector is always valid")
                .next()
                .expect("Will always find exactly one leaderboard position")
                .get(soup.parser())
                .expect("infallible")
                .inner_text(soup.parser());
            if position.trim() == "1)" {
                in_part_2 = !in_part_2;
            }
            let time_to_solve = get_leaderboard_time(
                day,
                &time
                    .query_selector(soup.parser(), ".leaderboard-time")
                    .expect("Selector is always valid")
                    .next()
                    .expect("Will always find exactly one")
                    .get(soup.parser())
                    .expect("infallible")
                    .inner_text(soup.parser()),
            );
            if in_part_2 {
                part_2_times.push(time_to_solve);
            } else {
                part_1_times.push(time_to_solve);
            }
        }
        if part_1_times.is_empty() {
            // No part 2 leaderboard; boards were read in backwards
            (part_1_times, part_2_times) = (part_2_times, part_1_times);
        }
        if part_1_times.len() == 100 && part_2_times.len() == 100 {
            // Both leaderboards are full, cache them
            let file = fs::File::create(&leaderboards).expect(
                "Failed to create leaderboard cache. Please check your permissions",
            );
            serde_json::to_writer(file, &(&part_1_times, &part_2_times)).expect(
                "Failed to write leaderboard cache. Please check your permissions",
            );
        }
        (part_1_times, part_2_times)
    }
}

pub(crate) fn practice_result_for(day: u32, year: i32) -> Vec<f64> {
    use chrono::Datelike;

    use crate::data::PRACTICE_DATA_DIR;

    let practice_data_dir = &*PRACTICE_DATA_DIR / year.to_string() / day.to_string();
    make(&practice_data_dir);
    let now = chrono::Utc::now();
    let file = practice_data_dir
        / format!("{:04}-{:02}-{:02}.json", now.year(), now.month(), now.day());
    if file.exists() {
        serde_json::from_reader(
            fs::File::open(file).expect("Opening practice data file should never fail"),
        )
        .expect("Failed to parse practice data")
    } else {
        vec![]
    }
}
