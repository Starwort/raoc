use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::time::Duration;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use crossterm::style::{style, Stylize};

use crate::data::{base_url, DATA_DIR, WAIT_TIME};
use crate::internal_util::{
    is_practice_mode,
    message_from_body,
    open_page,
    pretty_print,
    print_rank,
    strip_trailing_nl,
    Submissions,
};
use crate::sync_impl::internal_util::{
    calculate_practice_result,
    get,
    load_token_from_stdin,
    make,
    post,
};
use crate::sync_impl::wait;

/// Fetch and return the input for `day` of `year`.
///
/// If `--practice` is provided on the command line, pretend that today is the
/// day of the puzzle and wait for puzzle unlock accordingly. 'today' is
/// determined in UTC; from 0:00 to 5:00 UTC, this will block until 5:00 UTC.
/// After that, until 0:00 UTC the next day, input fetching will be instant.
///
/// All inputs are cached in the data directory.
///
/// # Panics
///
/// If the day and year do not correspond to a
/// valid puzzle.
#[must_use]
pub fn fetch(day: u32, year: i32, never_print: bool) -> String {
    assert!(year >= 2015, "Invalid year");
    assert!((1..=25).contains(&day), "Invalid day");
    let in_folder = &*DATA_DIR / year.to_string();
    make(&in_folder);
    let in_file = &in_folder / format!("{day}.in");

    let wait_for_unlock = |now: DateTime<Utc>, unlock: DateTime<Utc>| {
        if now < unlock {
            wait(
                "Waiting for puzzle unlock".yellow(),
                (unlock - now)
                    .to_std()
                    .unwrap_or_else(|_| unreachable!("Should always be positive")),
            );
            println!("{}", "Fetching input!".green());
            open_page(base_url(year, day).as_str());
            true
        } else {
            false
        }
    };

    if in_file.exists() {
        let should_print = if is_practice_mode() {
            let now = Utc::now();
            let unlock = Utc
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 5, 0, 0)
                .single()
                .unwrap_or_else(|| unreachable!("Today at 5AM is always valid"));
            wait_for_unlock(now, unlock)
        } else {
            false
        };
        let input = fs::read_to_string(&in_file).unwrap_or_else(|_| {
            fs::remove_file(in_file).expect("Removing the input file should not fail");
            fetch(day, year, true)
        });
        if should_print && !never_print && is_practice_mode() {
            println!("{input}");
        }
        input
    } else {
        let mut unlock = Utc
            .with_ymd_and_hms(year, 12, day, 5, 0, 0)
            .single()
            .unwrap_or_else(|| unreachable!("December days at 5AM are always valid"));
        let mut now = Utc::now();
        if is_practice_mode() {
            unlock = Utc
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 5, 0, 0)
                .single()
                .unwrap_or_else(|| unreachable!("Today at 5AM is always valid"));
        }
        if now < unlock {
            // On the first day, run a stray request to validate the user's token
            if day == 1 {
                let resp = get(&(base_url(year, day) + "/input"), true);
                if resp.status().is_client_error() {
                    load_token_from_stdin(
                        "Your token has expired. Please enter your new token.".red(),
                    );
                    return fetch(day, year, never_print);
                }
                now = Utc::now();
            }
            wait_for_unlock(now, unlock);
        }
        let resp = get(&(base_url(year, day) + "/input"), true);
        if !resp.status().is_success() {
            if resp.status().is_client_error() {
                load_token_from_stdin(
                    "Your token has expired. Please enter your new token.".red(),
                );
                return fetch(day, year, never_print);
            }
            panic!("Received bad response from server: {}", resp.status());
        }
        let input = strip_trailing_nl(
            resp.text()
                .unwrap_or_else(|_| unreachable!("Response should be text")),
        );
        fs::write(in_file, &input).unwrap_or_else(|_| {
            eprintln!(
                "{}",
                "Warning: Failed to cache input file. Please check your permissions."
                    .red()
            );
        });
        if !never_print {
            println!("{input}");
        }
        input
    }
}

/// Submit a solution.
///
/// Submissions are cached; submitting an already-submitted solution will return
/// the previous response.
///
/// # Panics
///
/// If the day and year do not correspond to a valid puzzle.
pub fn submit(day: u32, part: u32, year: i32, answer: impl Display) {
    submit_impl(day, part, year, answer.to_string());
}

fn submit_already_solved(
    solution: &str,
    answer: &str,
    day: u32,
    part: u32,
    year: i32,
    part_solutions: &HashMap<String, String>,
) {
    if is_practice_mode() {
        println!(
            "Submitting {} as the solution to part {}...",
            answer.blue(),
            style(part).blue()
        );
        return if solution == answer {
            calculate_practice_result(day, part, year);
        } else if part_solutions.contains_key(solution) {
            pretty_print(&part_solutions[solution]);
        } else {
            println!("{}", "That's not the right answer".red());
        };
    }
    println!(
        "Day {} part {} has already been solved.\nThe solution was: {}",
        style(day).blue(),
        style(part).blue(),
        solution.blue(),
    );
    print_rank(&part_solutions[solution]);
}

fn delay(msg: &str) -> bool {
    if msg.starts_with("You gave") {
        println!("{}", msg.red());
        let wait_match = WAIT_TIME.captures(msg).expect(
            "Found a message that appeared to be a submission delay, that wasn't a \
             submission delay",
        );
        let pause =
            60 * wait_match.get(1).map_or(0, |m| {
                m.as_str().parse::<u64>().expect("Failed to parse minutes")
            }) + wait_match.get(2).map_or(0, |m| {
                m.as_str().parse::<u64>().expect("Failed to parse seconds")
            });
        wait(
            format!(
                "{} {} {}",
                "Waiting".yellow(),
                style(pause).blue(),
                "seconds to retry...".yellow()
            ),
            Duration::from_secs(pause),
        );
        true
    } else {
        false
    }
}

fn submit_impl(day: u32, part: u32, year: i32, answer: String) {
    let submission_dir = &*DATA_DIR / year.to_string() / day.to_string();
    make(&submission_dir);
    let submissions = &submission_dir / "submissions.txt";
    let mut solutions = if submissions.exists() {
        serde_json::from_reader(
            fs::File::open(&submissions)
                .unwrap_or_else(|_| unreachable!("File should exist")),
        )
        .unwrap_or_else(|_| unreachable!("File should be valid"))
    } else {
        Submissions {
            part_1: HashMap::new(),
            part_2: HashMap::new(),
        }
    };
    let part_solutions = match part {
        1 => &mut solutions.part_1,
        2 => &mut solutions.part_2,
        _ => unreachable!("Part should be 1 or 2"),
    };

    let solution_file = &submission_dir / format!("{part}.solution");
    #[allow(clippy::map_entry)]
    if solution_file.exists() {
        let solution = fs::read_to_string(solution_file)
            .unwrap_or_else(|_| panic!("Solution file was corrupt"));
        submit_already_solved(&solution, &answer, day, part, year, part_solutions);
    } else if part_solutions.contains_key(&answer) {
        println!(
            "{} {} {} {} {}",
            "Solution: ".yellow(),
            answer.as_str().blue(),
            "to part".yellow(),
            style(part).blue(),
            "has already been submitted.\nResponse was:".yellow(),
        );
        pretty_print(part_solutions[&answer].as_str());
    } else {
        let mut msg;
        loop {
            println!(
                "Submitting {} as the solution to part {}...",
                answer.as_str().blue(),
                style(part).blue()
            );
            let resp = post(
                &(base_url(year, day) + "/answer"),
                true,
                HashMap::from([
                    ("level", part.to_string()),
                    ("answer", answer.to_string()),
                ]),
            );
            if !resp.status().is_success() {
                if resp.status().is_client_error() {
                    load_token_from_stdin(
                        "Your token has expired. Please enter your new token.".red(),
                    );
                    continue;
                }
                panic!("Received bad response from server: {}", resp.status());
            }

            let resp_text = resp
                .text()
                .unwrap_or_else(|_| unreachable!("Response should be text"));
            msg = message_from_body(&resp_text);
            if !delay(&msg) {
                break;
            }
        }
        if msg.starts_with("That's the") {
            print_rank(&msg);
            fs::write(solution_file, &answer).expect("Writing solution cache failed");
            calculate_practice_result(day, part, year);
            if part == 1 {
                open_page(&(base_url(year, day) + "#part2"));
            }
        } else {
            pretty_print(&msg);
        }

        part_solutions.insert(answer, msg);
        fs::write(submissions, serde_json::to_string(&solutions).unwrap())
            .expect("Writing submissions cache failed");
    }
}
