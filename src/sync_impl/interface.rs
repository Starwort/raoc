use std::fs;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use crossterm::style::Stylize;

use crate::data::{base_url, DATA_DIR};
use crate::internal_util::{is_practice_mode, open_page, strip_trailing_nl};
use crate::sync_impl::internal_util::{get, load_token_from_stdin, make};
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
