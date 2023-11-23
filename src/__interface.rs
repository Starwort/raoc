use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::io::{self, BufRead, Write};

use chrono::{Datelike, TimeZone, Utc};
use crossterm::style::{Color as Colour, Stylize};
use lazy_static::lazy_static;
use pathdiv::PathDiv;
use regex::Regex;

lazy_static! {
    pub(crate) static ref DATA_DIR: PathDiv = {
        let path = PathDiv::new()
            / dirs::home_dir().expect(concat!(
                "Failed to determine home directory.",
                " Please set the HOME environment variable.",
            ))
            / ".config"
            / "aoc_helper";

        if !path.exists() {
            fs::create_dir_all(&path).expect("Failed to create data directory.");
        }

        path
    };
}

lazy_static! {
    pub(crate) static ref DEFAULT_YEAR: i32 = Utc::now().year();
}

lazy_static! {
    pub(crate) static ref TODAY: u32 = Utc::now().day();
}

#[must_use]
pub fn day_url<Y: Display, D: Display>(year: Y, day: D) -> String {
    format!("https://adventofcode.com/{year}/day/{day}")
}

lazy_static! {
    pub(crate) static ref WAIT_TIME: Regex =
        Regex::new(r"You have (?:(\d+)m )?(\d+)s left to wait.").unwrap();
}

lazy_static! {
    pub(crate) static ref RANK: Regex =
        Regex::new(r"You (?:got|achieved) rank (\d+) on this star's leaderboard.")
            .unwrap();
}

#[cfg(feature = "blocking")]
use reqwest::blocking;

#[cfg(feature = "blocking")]
lazy_static! {
    pub(crate) static ref CLIENT: blocking::Client = blocking::Client::builder()
        .user_agent(concat!(
            "github.com/starwort/raoc v",
            env!("CARGO_PKG_VERSION"),
            " contact: Reddit u/starwort Discord @Starwort#6129",
        ))
        .build()
        .expect("Failed to build HTTP client.");
}

#[cfg(feature = "async")]
lazy_static! {
    pub(crate) static ref ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .user_agent(concat!(
            "github.com/starwort/raoc v",
            env!("CARGO_PKG_VERSION"),
            " contact: Reddit u/starwort Discord @Starwort#6129",
        ))
        .build()
        .expect("Failed to build async HTTP client.");
}

#[must_use]
pub fn get_token() -> String {
    let path = DATA_DIR.clone() / "token.txt";

    if let Ok(cookie) = fs::read_to_string(&path) {
        cookie
    } else {
        println!("Failed to load token from configuration file.");
        println!("Please enter your session token:");
        print!(">>> ");
        io::stdout().flush().expect("Failed to flush stdout.");
        let token = io::stdin()
            .lock()
            .lines()
            .next()
            .expect("Stdin closed before cookie provided.")
            .expect("Input was not valid UTF-8");
        if matches!(fs::write(path, &token), Err(_)) {
            eprintln!("Failed to write token to configuration file.");
        }
        token
    }
}

#[cfg(feature = "web")]
use reqwest::header::COOKIE;
#[cfg(feature = "web")]
use serde::Serialize as Payload;

#[cfg(feature = "blocking")]
pub(crate) fn get(url: &str) -> blocking::Response {
    let resp = CLIENT
        .get(url)
        .header(COOKIE, format!("session={}", get_token()))
        .send()
        .expect("Failed to send HTTP request.");

    if resp.status().is_client_error() {
        println!(
            "{}",
            "Your token has expired. Please enter your new token.".red(),
        );
        print!(">>> ");
        io::stdout().flush().expect("Failed to flush stdout.");
        let token = io::stdin()
            .lock()
            .lines()
            .next()
            .expect("Stdin closed before cookie provided.")
            .expect("Input was not valid UTF-8");
        match fs::write(DATA_DIR.clone() / "token.txt", token) {
            Ok(_) => get(url),
            Err(_) => Err(0).expect("Failed to write token to file."),
        }
    } else {
        resp
    }
}
#[cfg(feature = "async")]
pub(crate) async fn async_get(url: &str) -> reqwest::Response {
    loop {
        let resp = ASYNC_CLIENT
            .get(url)
            .header(COOKIE, format!("session={}", get_token()))
            .send()
            .await
            .expect("Failed to send HTTP request.");

        if resp.status().is_client_error() {
            println!(
                "{}",
                "Your token has expired. Please enter your new token.".red(),
            );
            print!(">>> ");
            io::stdout().flush().expect("Failed to flush stdout.");
            let token = io::stdin()
                .lock()
                .lines()
                .next()
                .expect("Stdin closed before cookie provided.")
                .expect("Input was not valid UTF-8");
            match fs::write(DATA_DIR.clone() / "token.txt", token) {
                Ok(_) => continue,
                Err(_) => Err(0).expect("Failed to write token to file."),
            }
        } else {
            return resp;
        }
    }
}

#[cfg(feature = "blocking")]
pub(crate) fn post(url: &str, data: impl Payload) -> blocking::Response {
    let resp = CLIENT
        .post(url)
        .form(&data)
        .header(COOKIE, format!("session={}", get_token()))
        .send()
        .expect("Failed to send HTTP request.");

    if resp.status().is_client_error() {
        println!(
            "{}",
            "Your token has expired. Please enter your new token.".red(),
        );
        print!(">>> ");
        io::stdout().flush().expect("Failed to flush stdout.");
        let token = io::stdin()
            .lock()
            .lines()
            .next()
            .expect("Stdin closed before cookie provided.")
            .expect("Input was not valid UTF-8");
        match fs::write(DATA_DIR.clone() / "token.txt", token) {
            Ok(_) => post(url, data),
            Err(_) => Err(0).expect("Failed to write token to file."),
        }
    } else {
        resp
    }
}
#[cfg(feature = "async")]
pub(crate) async fn async_post(url: &str, data: impl Payload) -> reqwest::Response {
    loop {
        let resp = ASYNC_CLIENT
            .post(url)
            .form(&data)
            .header(COOKIE, format!("session={}", get_token()))
            .send()
            .await
            .expect("Failed to send HTTP request.");

        if resp.status().is_client_error() {
            println!(
                "{}",
                "Your token has expired. Please enter your new token.".red(),
            );
            print!(">>> ");
            io::stdout().flush().expect("Failed to flush stdout.");
            let token = io::stdin()
                .lock()
                .lines()
                .next()
                .expect("Stdin closed before cookie provided.")
                .expect("Input was not valid UTF-8");
            match fs::write(DATA_DIR.clone() / "token.txt", token) {
                Ok(_) => continue,
                Err(_) => Err(0).expect("Failed to write token to file."),
            }
        } else {
            return resp;
        }
    }
}

use std::time::Duration;

#[cfg(feature = "blocking")]
pub(crate) fn wait<T: Display>(msg: T, duration: Duration) {
    use std::thread;

    println!("{msg}");
    thread::sleep(duration); // Todo: progress bar
}
#[cfg(feature = "async")]
pub(crate) async fn async_wait<T: Display>(msg: T, duration: Duration) {
    use std::future;
    use std::task::Poll::{Pending, Ready};
    use std::time::Instant;

    println!("{msg}");
    let start = Instant::now();
    future::poll_fn(|_| {
        if start.elapsed() < duration {
            Pending
        } else {
            Ready(())
        }
    })
    .await; // Todo: progress bar
}

#[cfg(feature = "blocking")]
pub(crate) fn work<T, U>(msg: &str, worker: impl FnOnce(U) -> T, data: U) -> T {
    println!("{msg}");
    worker(data) // Todo: spinner
}
#[cfg(feature = "async")]
use std::future::Future;
#[cfg(feature = "async")]
pub(crate) async fn async_work<T, U, F: Future<Output = T>>(
    msg: &str,
    worker: impl FnOnce(U) -> F,
    data: U,
) -> T {
    println!("{msg}");
    worker(data).await // Todo: spinner
}

/// Open the page, if the user hasn't opted out.
fn open_page(url: &str) {
    if !(DATA_DIR.clone() / ".nobrowser").exists() {
        webbrowser::open(url).expect("Failed to open web browser.");
    }
}

/// Analyse the response from the server and print it.
fn pretty_print(message: &str) {
    if message.starts_with("That's the") {
        println!("{}", message.green());
    } else if message.starts_with("You don't") {
        println!("{}", message.yellow());
    } else if message.starts_with("That's not") {
        println!("{}", message.red());
    } else {
        eprintln!("Failed to classify message");
        println!("{message}");
    }
}

#[cfg(feature = "blocking")]
/// Fetch and return the input for `day` of `year`.
///
/// All inputs are cached in the data directory.
#[must_use]
pub fn fetch(day: u32, year: i32, never_print: bool) -> String {
    let str_day = day.to_string();
    let str_year = year.to_string();
    let year_path = DATA_DIR.clone() / &str_year;
    fs::create_dir_all(&year_path).expect("Failed to create year directory.");
    let input_path = {
        let mut path = year_path / &str_day;
        path.set_extension(".in");
        path
    };
    if input_path.exists() {
        match fs::read_to_string(&input_path) {
            Ok(input) => {
                return input;
            },
            Err(_) => {
                eprintln!("Cache corrupt; redownloading input.");
            },
        }
    }
    let unlock = match Utc.with_ymd_and_hms(year, 12, day, 5, 0, 0) {
        chrono::LocalResult::Single(date) => date,
        _ => Err(0).expect("Year or day is invalid."),
    };
    let mut now = Utc::now();
    if now < unlock {
        // On the first day, run an extra request to validate the user's token.
        let _ = get(&(day_url(1, 2015) + "/input"));
        now = Utc::now();
    }
    let now = now;
    wait(
        "Waiting for puzzle unlock...".yellow(),
        (unlock - now)
            .to_std()
            .expect("Duration failed to convert."),
    );
    println!("{}", "Fetching input!".green());
    let url = day_url(day, year);
    open_page(&url);
    let resp = get(&(url + "/input"));
    let mut input = resp.text().expect("Failed to read response body.");
    input.truncate(input.trim_end_matches('\n').len());
    if fs::write(input_path, &input).is_err() {
        eprintln!("Failed to write input to cache.");
    }
    if !never_print {
        println!("{input}");
    }
    input
}

#[cfg(feature = "async")]
/// Fetch and return the input for `day` of `year`.
///
/// All inputs are cached in the data directory.
///
/// # Panics
///
/// This function will panic if the year or day is invalid, if creating the
/// data directory fails, or if the Advent of Code server sends us an empty
/// response.
pub async fn async_fetch(day: u32, year: i32, never_print: bool) -> String {
    let str_day = day.to_string();
    let str_year = year.to_string();
    let year_path = DATA_DIR.clone() / &str_year;
    fs::create_dir_all(&year_path).expect("Failed to create year directory.");
    let input_path = {
        let mut path = year_path / &str_day;
        path.set_extension(".in");
        path
    };
    if input_path.exists() {
        match fs::read_to_string(&input_path) {
            Ok(input) => {
                return input;
            },
            Err(_) => {
                eprintln!("Cache corrupt; redownloading input.");
            },
        }
    }
    assert!(day <= 25, "Day is invalid.");
    let chrono::LocalResult::Single(unlock) =
        Utc.with_ymd_and_hms(year, 12, day, 5, 0, 0)
    else {
        panic!("Year or day is invalid.")
    };
    let mut now = Utc::now();
    let must_wait = now < unlock;
    if must_wait {
        // On the first day, run an extra request to validate the user's token.
        let _ = async_get(&(day_url(1, 2015) + "/input")).await;
        now = Utc::now();
        async_wait(
            "Waiting for puzzle unlock...".yellow(),
            (unlock - now)
                .to_std()
                .expect("Duration failed to convert."),
        )
        .await;
    }
    println!("{}", "Fetching input!".green());
    let url = day_url(day, year);
    if must_wait {
        open_page(&url);
    }
    let resp = async_get(&(url + "/input")).await;
    let mut input = resp.text().await.expect("Failed to read response body.");
    input.truncate(input.trim_end_matches('\n').len());
    if fs::write(input_path, &input).is_err() {
        eprintln!("Failed to write input to cache.");
    }
    if !never_print && must_wait {
        println!("{input}");
    }
    input
}

fn print_rank(msg: &str) {
    if let Some(r#match) = RANK.find(msg) {
        println!("{}", r#match.as_str().with(Colour::AnsiValue(220)));
    }
}

fn load_submissions(
    submission_dir: &PathDiv,
) -> HashMap<String, HashMap<String, String>> {
    fs::create_dir_all(submission_dir).expect("Failed to create submission directory.");
    let submissions_file = submission_dir.clone() / "submissions.json";
    if submissions_file.exists() {
        serde_json::from_str(
            &fs::read_to_string(&submissions_file)
                .expect("Failed to read submissions file."),
        )
        .expect("Failed to parse submissions file.")
    } else {
        HashMap::from([
            ("1".to_string(), HashMap::new()),
            ("2".to_string(), HashMap::new()),
        ])
    }
}

fn update_submissions(
    submission_dir: PathDiv,
    mut submissions: HashMap<String, HashMap<String, String>>,
    part: &str,
    answer: String,
    msg: String,
) {
    submissions
        .get_mut(part)
        .expect("Failed to get part submissions")
        .insert(answer, msg);
    let submissions_file = submission_dir / "submissions.json";
    fs::write(
        submissions_file,
        serde_json::to_string(&submissions).expect("Failed to serialize submissions."),
    )
    .expect("Failed to write submissions file.");
}

#[cfg(feature = "web")]
fn message_from_body(body: &str) -> String {
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

#[cfg(feature = "blocking")]
/// Submit a solution.
///
/// Submissions are cached; submitting an already-submitted solution will return
/// the previous response.
///
/// # Panics
///
/// This function will panic if the Advent of Code server sends us an empty
/// response or if writing to disk fails.
pub fn submit(day: u32, part: u32, year: i32, answer: impl Display) {
    let str_day = day.to_string();
    let str_year = year.to_string();
    let str_part = part.to_string();
    let str_answer = answer.to_string();
    let submission_dir = DATA_DIR.clone() / &str_year / &str_day;
    let submissions = load_submissions(&submission_dir);
    let mut solution_file = submission_dir.clone() / &str_part;
    solution_file.set_extension("solution");
    if solution_file.exists() {
        let solution =
            fs::read_to_string(&solution_file).expect("Failed to read solution file.");
        println!(
            "Day {} part {} has already been solved.",
            str_day.as_str().blue(),
            str_part.as_str().blue()
        );
        println!("The solution was: {}", solution.as_str().blue());
        print_rank(&submissions[&str_part][&solution]);
    } else if submissions[&str_part].contains_key(&str_answer) {
        println!(
            "{} {} {} {} {}",
            "Solution:".yellow(),
            str_answer.as_str().blue(),
            "to part".yellow(),
            str_part.as_str().blue(),
            "has already been submitted.".yellow(),
        );
        println!("{}", "Response was".yellow());
        pretty_print(&submissions[&str_part][&str_answer]);
    } else {
        let msg = loop {
            println!(
                "Submitting {} as the solution to part {}...",
                str_answer.as_str().blue(),
                str_part.as_str().blue(),
            );
            let resp = post(
                &(day_url(year, day) + "/answer"),
                [("level", &str_part), ("answer", &str_answer)],
            );
            let body = resp.text().expect("Failed to read response body.");
            let msg = message_from_body(&body);
            if msg.starts_with("You gave") {
                println!("{}", msg.as_str().red());
                let wait_match = WAIT_TIME
                    .captures(&msg)
                    .expect("Failed to parse wait time.");
                let pause = 60
                    * wait_match
                        .get(1)
                        .map_or("0", |m| m.as_str())
                        .parse::<u64>()
                        .expect("Parsing integer failed")
                    + wait_match
                        .get(2)
                        .map_or("0", |m| m.as_str())
                        .parse::<u64>()
                        .expect("Parsing integer failed");
                wait(
                    format!(
                        "{} {} {}",
                        "Waiting".yellow(),
                        pause.to_string().blue(),
                        "seconds to retry...".yellow()
                    )
                    .as_str(),
                    Duration::from_secs(pause),
                );
            } else {
                break msg;
            }
        };
        if msg.starts_with("That's the") {
            pretty_print(&msg);
            print_rank(&msg);
            fs::write(solution_file, &str_answer)
                .expect("Failed to write solution file to cache.");
            if part == 1 {
                open_page(&(day_url(year, day) + "#part2"));
            }
        } else {
            pretty_print(&msg);
        }

        update_submissions(submission_dir, submissions, &str_part, str_answer, msg);
    }
}
#[cfg(feature = "async")]
/// Submit a solution.
///
/// Submissions are cached; submitting an already-submitted solution will return
/// the previous response.
///
/// # Panics
///
/// This function will panic if the Advent of Code server sends us an empty
/// response or if writing to disk fails.
pub async fn async_submit(day: u32, part: u32, year: i32, answer: impl Display) {
    let str_day = day.to_string();
    let str_year = year.to_string();
    let str_part = part.to_string();
    let str_answer = answer.to_string();
    let submission_dir = DATA_DIR.clone() / &str_year / &str_day;
    let submissions = load_submissions(&submission_dir);
    let mut solution_file = submission_dir.clone() / &str_part;
    solution_file.set_extension("solution");
    if solution_file.exists() {
        let solution =
            fs::read_to_string(&solution_file).expect("Failed to read solution file.");
        println!(
            "Day {} part {} has already been solved.",
            str_day.as_str().blue(),
            str_part.as_str().blue()
        );
        println!("The solution was: {}", solution.as_str().blue());
        print_rank(&submissions[&str_part][&solution]);
    } else if submissions[&str_part].contains_key(&str_answer) {
        println!(
            "{} {} {} {} {}",
            "Solution:".yellow(),
            str_answer.as_str().blue(),
            "to part".yellow(),
            str_part.as_str().blue(),
            "has already been submitted.".yellow(),
        );
        println!("{}", "Response was".yellow());
        pretty_print(&submissions[&str_part][&str_answer]);
    } else {
        let msg = loop {
            println!(
                "Submitting {} as the solution to part {}...",
                str_answer.as_str().blue(),
                str_part.as_str().blue(),
            );
            let resp = async_post(
                &(day_url(year, day) + "/answer"),
                [("level", &str_part), ("answer", &str_answer)],
            )
            .await;
            let body = resp.text().await.expect("Failed to read response body.");
            let msg = message_from_body(&body);
            if msg.starts_with("You gave") {
                println!("{}", msg.as_str().red());
                let wait_match = WAIT_TIME
                    .captures(&msg)
                    .expect("Failed to parse wait time.");
                let pause = 60
                    * wait_match
                        .get(1)
                        .map_or("0", |m| m.as_str())
                        .parse::<u64>()
                        .expect("Parsing integer failed")
                    + wait_match
                        .get(2)
                        .map_or("0", |m| m.as_str())
                        .parse::<u64>()
                        .expect("Parsing integer failed");
                async_wait(
                    format!(
                        "{} {} {}",
                        "Waiting".yellow(),
                        pause.to_string().blue(),
                        "seconds to retry...".yellow()
                    )
                    .as_str(),
                    Duration::from_secs(pause),
                )
                .await;
            } else {
                break msg;
            }
        };
        if msg.starts_with("That's the") {
            pretty_print(&msg);
            print_rank(&msg);
            fs::write(solution_file, &str_answer)
                .expect("Failed to write solution file to cache.");
            if part == 1 {
                open_page(&(day_url(year, day) + "#part2"));
            }
        } else {
            pretty_print(&msg);
        }

        update_submissions(submission_dir, submissions, &str_part, str_answer, msg);
    }
}

#[cfg(feature = "blocking")]
/// Finish Advent of Code for the year. Day 25 part 2 is special, as there is no
/// puzzle; instead, it is a free star awarded for completing every other
/// puzzle. This function will submit 0 as the answer to day 25 part 2, just
/// like the website would.
///
/// # Panics
///
/// This function will panic if the Advent of Code server sends us an empty
/// response
pub fn submit_25(year: &str) {
    println!(
        "{} {}{}",
        "Finishing Advent of Code".green(),
        year.blue(),
        "!".green()
    );
    let resp = post(
        &(day_url(year, 25) + "/answer"),
        [("level", "2"), ("answer", "0")],
    );
    println!("Response from the server:");
    println!(
        "{}",
        message_from_body(&resp.text().expect("Failed to read response body."),)
    );
}
#[cfg(feature = "async")]
/// Finish Advent of Code for the year. Day 25 part 2 is special, as there is no
/// puzzle; instead, it is a free star awarded for completing every other
/// puzzle. This function will submit 0 as the answer to day 25 part 2, just
/// like the website would.
///
/// # Panics
///
/// This function will panic if the Advent of Code server sends us an empty
/// response
pub async fn async_submit_25(year: &str) {
    println!(
        "{} {}{}",
        "Finishing Advent of Code".green(),
        year.blue(),
        "!".green()
    );
    let resp = async_post(
        &(day_url(year, 25) + "/answer"),
        [("level", "2"), ("answer", "0")],
    )
    .await;
    println!("Response from the server:");
    println!(
        "{}",
        message_from_body(&resp.text().await.expect("Failed to read response body."))
    );
}

#[cfg(feature = "blocking")]
// should not be possible for this function to ever panic
#[allow(clippy::missing_panics_doc)]
/// Run the solution only if we have not already submitted a correct answer.
pub fn lazy_submit<T: MaybeDisplay, U>(
    day: u32,
    part: u32,
    year: i32,
    solution: impl FnOnce(U) -> T,
    data: U,
) {
    let submission_dir = DATA_DIR.clone() / &year.to_string() / &day.to_string();
    if day == 25 && part == 2 && (submission_dir.clone() / "1.solution").exists() {
        submit_25(&year.to_string());
    } else if (submission_dir.clone() / &(part.to_string() + ".solution")).exists() {
        let submissions = load_submissions(&submission_dir);
        let str_solution =
            fs::read_to_string(submission_dir / &(part.to_string() + ".solution"))
                .expect("Failed to read solution file.");
        println!(
            "Day {} part {} has already been solved.",
            day.to_string().blue(),
            part.to_string().blue()
        );
        println!("The solution was: {}", str_solution.as_str().blue());
        print_rank(&submissions[&part.to_string()][&str_solution]);
    } else {
        let answer = work(
            &format!(
                "{} {} {}",
                "Running part".yellow(),
                part.to_string().blue(),
                "solution...".yellow()
            ),
            solution,
            data,
        );
        if let Some(answer) = answer.as_display() {
            submit(day, part, year, answer);
        }
    }
}
