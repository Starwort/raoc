#![feature(negative_impls, auto_traits)]
#[cfg(feature = "async")]
mod async_impl;
#[cfg(feature = "async")]
#[cfg_attr(
    feature = "sync",
    allow(ambiguous_glob_reexports, clippy::useless_attribute)
)]
#[allow(unused_imports)]
pub use async_impl::{
    fetch as async_fetch,
    submit as async_submit,
    wait as async_wait,
    work as async_work,
    *,
};
#[cfg(feature = "sync")]
mod sync_impl;
#[cfg(feature = "sync")]
#[allow(unused_imports)]
pub use sync_impl::{
    fetch as sync_fetch,
    submit as sync_submit,
    wait as sync_wait,
    work as sync_work,
    *,
};
mod data;
mod internal_util;
mod maybe_display;
pub use maybe_display::MaybeDisplay;
