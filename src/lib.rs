#![feature(must_not_suspend, result_flattening)]
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    explicit_outlives_requirements,
    let_underscore_drop,
    missing_copy_implementations,
    missing_debug_implementations,
    must_not_suspend,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    unreachable_pub,
    unused_results,
    variant_size_differences
)]
#![allow(clippy::new_without_default)]

pub mod config;

pub mod http;

mod password;
