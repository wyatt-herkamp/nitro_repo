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
pub mod seed;
