//! Shared API responses.
//!
//! The types themselves moved to `nr-web-core` so repository crates can return them;
//! `RepositoryNotFound` went to `nr-repository`, next to the lookup that produces it. Re-exported
//! here so `app::responses::` still resolves.

pub use nr_repository::RepositoryNotFound;
pub use nr_web_core::responses::*;
