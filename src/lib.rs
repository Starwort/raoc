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
    get_sample_input as async_get_sample_input,
    lazy_submit as async_lazy_submit,
    lazy_submit_part as async_lazy_submit_part,
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
    get_sample_input as sync_get_sample_input,
    lazy_submit as sync_lazy_submit,
    lazy_submit_part as sync_lazy_submit_part,
    submit as sync_submit,
    wait as sync_wait,
    work as sync_work,
    *,
};
mod data;
mod internal_util;
mod maybe_display;
pub use maybe_display::MaybeDisplay;

#[cfg(all(feature = "simd", not(feature = "web")))]
compile_error!(
    "Cannot enable SIMD without enabling either the 'sync' or 'async' feature."
);

#[cfg(all(feature = "web", not(any(feature = "sync", feature = "async"))))]
compile_error!(
    "Please do not enable the 'web' feature manually. It is an internal feature used \
     by the 'sync' and 'async' features - please enable one of those instead."
);
