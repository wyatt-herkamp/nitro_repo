//! The nitro_repo server.
//!
//! This crate was a binary with no library target, which meant nothing outside `main.rs` could
//! reach the router, the state, or the deploy handlers — so there was no way to write an
//! integration test that exercised a real request. `tests/` now builds the same router `start`
//! serves and drives it in-process.
//!
//! `main.rs` is the CLI; everything it needs lives here.

pub mod app;
pub mod logging;
pub mod repository;
pub mod seed;

/// `error` and `utils` now live in `nr-web-core`.
///
/// Re-exported under their old paths so the several hundred `crate::utils::` and `crate::error::`
/// call sites in this crate did not have to move in the same commit the files did. The re-exports
/// go away once everything is in its final crate.
pub use nr_web_core::{error, utils};
