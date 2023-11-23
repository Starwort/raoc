use std::fs;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use crossterm::style::Stylize;

#[cfg(feature = "async")]
use crate::async_wait;
use crate::data::{base_url, DATA_DIR};
#[cfg(feature = "async")]
use crate::internal_util::{async_get, async_load_token_from_stdin};
#[cfg(feature = "sync")]
use crate::internal_util::{get, load_token_from_stdin};
use crate::internal_util::{is_practice_mode, make, open_page, strip_trailing_nl};
#[cfg(feature = "sync")]
use crate::wait;

#[cfg(feature = "sync")]