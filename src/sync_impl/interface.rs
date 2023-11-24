use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::time::Duration;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use crossterm::style::{style, Stylize};
#[cfg(feature = "pretty_assertions")]
use pretty_assertions::assert_eq;

use super::internal_util::{
    calculate_practice_result,
    get,
    load_token_from_stdin,
    make,
    post,
    practice_result_for,
    wait,
    work,
};
use crate::data::{base_url, DATA_DIR, WAIT_TIME};
use crate::internal_util::{
    is_practice_mode,
    message_from_body,
    must_run_solutions,
    open_page,
    pretty_print,
    print_rank,
    strip_trailing_nl,
    Submissions,
};
use crate::MaybeDisplay;

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
fn submit_25(year: &str) {
    let resp = loop {
        println!(
            "{} {}{}",
            "Finishing Advent of Code".green(),
            year.blue(),
            '!'.green(),
        );
        let resp = post(
            &(base_url(year, 25) + "/answer"),
            true,
            HashMap::from([("level", "2"), ("answer", "0")]),
        );
        if resp.status().is_success() {
            break resp;
        } else if resp.status().is_client_error() {
            load_token_from_stdin(
                "Your token has expired. Please enter your new token.".red(),
            );
        } else {
            panic!("Received bad response from server: {}", resp.status());
        }
    };

    println!("Response from the server:");
    println!(
        "{}",
        message_from_body(&resp.text().expect("Response should be text")),
    );
}

/// Run the functions only if we haven't seen a solution.
///
/// Will also run solutions if `--force-run` or `--practice` is passed on the
/// command line.
///
/// The solution for part 2 will be ignored if day is 25.
pub fn lazy_submit<U, S1: MaybeDisplay, S2: MaybeDisplay>(
    day: u32,
    year: i32,
    solution_part_1: impl FnOnce(U) -> S1,
    solution_part_2: impl FnOnce(U) -> S2,
    mut parse_raw: impl FnMut(&str) -> U,
) {
    lazy_submit_part(day, year, 1, solution_part_1, &mut parse_raw);
    lazy_submit_part(day, year, 2, solution_part_2, &mut parse_raw);
}

/// Run the function only if we haven't seen a solution.
///
/// Will also run solution if `--force-run` or `--practice` is passed on the
/// command line.
///
/// If the day is 25 and the part is 2, will ignore the solution.
pub fn lazy_submit_part<U, M: MaybeDisplay>(
    day: u32,
    year: i32,
    part: u32,
    solution_part_1: impl FnOnce(U) -> M,
    parse_raw: impl FnOnce(&str) -> U,
) {
    let submission_dir = &*DATA_DIR / year.to_string() / day.to_string();
    make(&submission_dir);
    if day == 25 && part == 2 {
        // don't try to submit part 2 if part 1 isn't solved
        if (&submission_dir / "1.solution").exists() {
            submit_25(&year.to_string());
        } else {
            return;
        }
    }
    let solution_file = &submission_dir / format!("{part}.solution");
    if !solution_file.exists()
        || must_run_solutions()
        || (is_practice_mode()
            && practice_result_for(day, year).1.len() < part as usize)
    {
        let answer = work(
            format!(
                "{} {} {}",
                "Running part".yellow(),
                style(part).blue(),
                "solution".yellow(),
            ),
            || {
                let raw = fetch(day, year, false);
                solution_part_1(parse_raw(&raw))
            },
        )
        .into_solution();
        if let Some(answer) = answer {
            submit(day, part, year, answer);
        }
    } else {
        // load cached solutions
        let submissions = submission_dir / "submissions.json";
        let solutions: Submissions = serde_json::from_str(
            &fs::read_to_string(submissions).unwrap_or_else(|_| {
                unreachable!(
                    "Failed to read submission cache, which must exist as this part \
                     has already been solved"
                )
            }),
        )
        .unwrap_or_else(|_| unreachable!("Failed to parse submission cache"));

        let solution = fs::read_to_string(solution_file)
            .unwrap_or_else(|_| unreachable!("Solution file was corrupt"));
        let response = match part {
            1 => &solutions.part_1[&solution],
            2 => &solutions.part_2[&solution],
            _ => unreachable!("Part should be 1 or 2"),
        };
        println!(
            "Day {} part {} has already been solved.\nThe solution was {}",
            style(day).blue(),
            style(part).blue(),
            solution.blue(),
        );
        print_rank(response);
    }
}
