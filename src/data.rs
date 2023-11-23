use std::fmt::Display;
use std::fs;

use crossterm::style::Color as Colour;
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
    pub(crate) static ref PRACTICE_DATA_DIR: PathDiv = {
        let path = PathDiv::new()
            / dirs::home_dir().expect(concat!(
                "Failed to determine home directory.",
                " Please set the HOME environment variable.",
            ))
            / ".config"
            / "aoc_helper"
            / "practice";

        if !path.exists() {
            fs::create_dir_all(&path).expect("Failed to create data directory.");
        }

        path
    };
    pub(crate) static ref TOKEN_FILE: PathDiv = &*DATA_DIR / "token.txt";
    pub(crate) static ref WAIT_TIME: Regex =
        Regex::new(r"You have (?:(\d+)m )?(\d+)s left to wait.").expect("Infallible");
    pub(crate) static ref RANK: Regex =
        Regex::new(r"You (?:got|achieved) rank (\d+) on this star's leaderboard.")
            .expect("Infallible");
}
pub(crate) const USER_AGENT: &str = concat!(
    "github.com/starwort/raoc v",
    env!("CARGO_PKG_VERSION"),
    " contact: Discord @starwort",
    " Github https://github.com/Starwort/raoc/issues",
);
pub(crate) const GOLD: Colour = Colour::Rgb {
    r: 255,
    g: 215,
    b: 135,
};

pub(crate) fn leaderboard_url(year: impl Display, day: impl Display) -> String {
    format!("https://adventofcode.com/{year}/leaderboard/day/{day}")
}
pub(crate) fn base_url(year: impl Display, day: impl Display) -> String {
    format!("https://adventofcode.com/{year}/day/{day}")
}
