#![feature(negative_impls, auto_traits)]
#[cfg(feature = "async")]
mod async_impl;
#[cfg(feature = "async")]
pub use async_impl::{fetch as async_fetch, wait as async_wait, work as async_work, *};
#[cfg(feature = "sync")]
mod sync_impl;
#[cfg(feature = "sync")]
pub use sync_impl::{fetch as sync_fetch, wait as sync_wait, work as sync_work, *};
mod data;
pub mod interface;
mod internal_util;
mod maybe_display;
pub use maybe_display::MaybeDisplay;
