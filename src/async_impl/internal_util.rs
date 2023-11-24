use std::fmt::Display;
use std::future::Future;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::{Datelike, NaiveDate, Utc};
use crossterm::style::{style, Stylize};
use lazy_static::lazy_static;
use pathdiv::PathDiv;
use reqwest::{header, Client, Response};
use tokio::{fs, time};

use crate::data::{
    leaderboard_url,
    DATA_DIR,
    GOLD,
    PRACTICE_DATA_DIR,
    TOKEN_FILE,
    USER_AGENT,
};
use crate::internal_util::{
    format_time,
    get_leaderboard_time,
    is_practice_mode,
    strip_trailing_nl,
};

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

pub(crate) async fn post(
    url: &str,
    authenticate: bool,
    data: impl serde::Serialize,
) -> Response {
    if authenticate {
        CLIENT.post(url).header(header::COOKIE, get_cookie().await)
    } else {
        CLIENT.post(url)
    }
    .form(&data)
    .send()
    .await
    .expect("Advent of Code sent back a bad response, or the network is down.")
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
        let soup =
            tl::parse(&leaderboard_page, tl::ParserOptions::new().track_classes())
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
            fs::write(
                &leaderboards,
                serde_json::to_string(&(&part_1_times, &part_2_times))
                    .expect("Serialising should never fail"),
            )
            .await
            .expect("Failed to write leaderboard cache. Please check your permissions");
        }
        (part_1_times, part_2_times)
    }
}

pub(crate) async fn practice_result_for(day: u32, year: i32) -> (PathDiv, Vec<f64>) {
    let practice_data_dir = &*PRACTICE_DATA_DIR / year.to_string() / day.to_string();
    make(&practice_data_dir).await;
    let now = Utc::now();
    let file = practice_data_dir
        / format!("{:04}-{:02}-{:02}.json", now.year(), now.month(), now.day());
    if file.exists() {
        let data = String::new();
        fs::read_to_string(&file)
            .await
            .expect("Reading practice data should not fail");
        (
            file,
            serde_json::from_str(&data).expect("Failed to parse practice data"),
        )
    } else {
        (file, vec![])
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

pub(crate) async fn calculate_practice_result(day: u32, part: u32, year: i32) {
    if !is_practice_mode() {
        return;
    }
    let now = Utc::now();
    #[allow(deprecated)]
    let solve_time = now
        .signed_duration_since(
            NaiveDate::from_ymd(now.year(), now.month(), now.day())
                .and_hms(5, 0, 0)
                .and_utc(),
        )
        .to_std()
        .expect("Should never be negative")
        .as_secs_f64();
    let (file, mut data) = practice_result_for(day, year).await;
    data.push(solve_time);
    fs::write(
        file,
        serde_json::to_string(&data).expect("Serialising results should be infallible"),
    )
    .await
    .expect("Saving practice results failed");
    report_practice_result(day, part, year, solve_time).await;
}

async fn estimate_practice_rank(
    day: u32,
    part: u32,
    year: i32,
    solve_time: f64,
) -> Option<(usize, usize, usize)> {
    let leaderboard = load_leaderboard_times(day, year).await;
    let leaderboard = match part {
        1 => leaderboard.0,
        2 => leaderboard.1,
        _ => panic!("part was neither 1 nor 2"),
    };
    let truncated_solve_time = solve_time.trunc();
    let best_possible_rank =
        leaderboard.partition_point(|&opp_time| opp_time < truncated_solve_time) + 1;
    let worst_possible_rank =
        leaderboard.partition_point(|&opp_time| opp_time < solve_time) + 1;
    if best_possible_rank > 100 {
        None
    } else {
        let span = worst_possible_rank - best_possible_rank;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let approx = best_possible_rank + (span as f64 * solve_time.fract()) as usize;
        Some((approx, best_possible_rank, worst_possible_rank))
    }
}

async fn report_practice_result(day: u32, part: u32, year: i32, solve_time: f64) {
    println!(
        "{} {}{}",
        "You solved the puzzle in".green(),
        format_time(solve_time).blue(),
        '!'.green(),
    );

    let result = estimate_practice_rank(day, part, year, solve_time).await;
    match result {
        None => {
            println!(
                "{}",
                "You would not have achieved a leaderboard position.".yellow()
            );
        },
        Some((_approx, best, worst)) if best == worst => {
            println!(
                "{} {}{}",
                "You would have achieved rank".with(GOLD),
                style(best).with(GOLD),
                '!'.with(GOLD)
            );
        },
        Some((approx, best, worst)) => {
            println!(
                "{} {} {}{} {} {}{}{}",
                "You would have achieved approximately rank".with(GOLD),
                style(approx).with(GOLD),
                '('.with(GOLD),
                style(best).with(GOLD),
                "to".with(GOLD),
                style(if worst > 100 { 100 } else { worst }).with(GOLD),
                if worst > 100 { "+" } else { "" }.with(GOLD),
                ")!".with(GOLD),
            );
        },
    }
}
