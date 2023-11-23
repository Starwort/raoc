use std::fmt::Display;
use std::future::Future;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::Datelike;
use crossterm::style::{style, Stylize};
use lazy_static::lazy_static;
use reqwest::{header, Client, Response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{fs, time};

use crate::data::{
    leaderboard_url,
    DATA_DIR,
    PRACTICE_DATA_DIR,
    TOKEN_FILE,
    USER_AGENT,
};
use crate::internal_util::{get_leaderboard_time, strip_trailing_nl};

pub(crate) async fn load_token_from_stdin(why: impl Display) -> String {
    eprintln!("{why}");
    eprint!(">>> ");
    let mut token = String::new();
    io::stdin()
        .read_line(&mut token)
        .expect("Failed to read token.");
    token = strip_trailing_nl(token);
    fs::write(&*TOKEN_FILE, &token)
        .await
        .expect("Failed to write token file. Check your permissions.");
    token
}

/// Wait the specified time, displaying a countdown, a spinner, and a message.
pub async fn wait(msg: impl Display, time: Duration) {
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
        time::sleep(Duration::from_millis(100)).await;
        time_left = end - Instant::now();
    }
    eprintln!();
}

/// Run the given worker function, displaying a message, spinner, and elapsed
/// timer.
pub async fn work<T>(msg: impl Display, worker: impl Future<Output = T>) -> T {
    tokio::select! {
        result = worker => {
            eprintln!();
            result
        },
        () = async {
            let start = Instant::now();
            loop {
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
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        } => {
            unreachable!("Spinner future should never complete")
        }
    }
}

/// Make a directory, if it doesn't exist.
pub(crate) async fn make(dir: &Path) {
    if !dir.exists() {
        fs::create_dir_all(dir)
            .await
            .expect("Failed to create data directory.");
    }
}

lazy_static! {
    static ref CLIENT: Client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to build reqwest client.");
}

pub(crate) async fn get(url: &str, authenticate: bool) -> Response {
    if authenticate {
        CLIENT.get(url).header(header::COOKIE, get_cookie().await)
    } else {
        CLIENT.get(url)
    }
    .send()
    .await
    .expect("Advent of Code sent back a bad response, or the network is down.")
}

pub(crate) async fn get_text(url: &str, authenticate: bool) -> String {
    get(url, authenticate)
        .await
        .text()
        .await
        .expect("Advent of Code sent back a bad response")
}

pub(crate) async fn post(url: &str, authenticate: bool) -> Response {
    if authenticate {
        CLIENT.post(url).header(header::COOKIE, get_cookie().await)
    } else {
        CLIENT.post(url)
    }
    .send()
    .await
    .expect("Advent of Code sent back a bad response, or the network is down.")
}

pub(crate) async fn post_text(url: &str, authenticate: bool) -> String {
    post(url, authenticate)
        .await
        .text()
        .await
        .expect("Advent of Code sent back a bad response")
}

pub(crate) async fn load_leaderboard_times(
    day: u32,
    year: i32,
) -> (Vec<f64>, Vec<f64>) {
    let day_dir = &*DATA_DIR / year.to_string() / day.to_string();
    make(&day_dir).await;

    let leaderboards = day_dir / "leaderboards.json";
    if leaderboards.exists() {
        let data = tokio::fs::read_to_string(&leaderboards)
            .await
            .expect("Failed to read leaderboards file.");
        serde_json::from_str(&data).expect("Failed to parse leaderboard cache.")
    } else {
        let leaderboard_page = get_text(&leaderboard_url(year, day), false).await;
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
            let mut file = tokio::fs::File::create(&leaderboards).await.expect(
                "Failed to create leaderboard cache. Please check your permissions",
            );
            file.write_all(
                &serde_json::to_vec(&(&part_1_times, &part_2_times))
                    .expect("Serialising should never fail"),
            )
            .await
            .expect("Failed to write leaderboard cache. Please check your permissions");
        }
        (part_1_times, part_2_times)
    }
}

pub(crate) async fn practice_result_for(day: u32, year: i32) -> Vec<f64> {
    let practice_data_dir = &*PRACTICE_DATA_DIR / year.to_string() / day.to_string();
    make(&practice_data_dir).await;
    let now = chrono::Utc::now();
    let file = practice_data_dir
        / format!("{:04}-{:02}-{:02}.json", now.year(), now.month(), now.day());
    if file.exists() {
        let mut data = String::new();
        fs::File::open(file)
            .await
            .expect("Opening practice data file should never fail")
            .read_to_string(&mut data)
            .await
            .expect("Reading the file should not fail");
        serde_json::from_str(&data).expect("Failed to parse practice data")
    } else {
        vec![]
    }
}

pub(crate) async fn get_cookie() -> String {
    let token = if TOKEN_FILE.exists() {
        strip_trailing_nl(
            fs::read_to_string(&*TOKEN_FILE)
                .await
                .expect("Failed to read token file."),
        )
    } else {
        load_token_from_stdin(
            "Could not find configuration file. Please enter your token",
        )
        .await
    };
    format!("session={token}")
}
