//! HTTP plumbing shared by the Nitro Repo server and by every repository type.
//!
//! Web vocabulary, not repository vocabulary: response building, the error trait every handler
//! funnels through, request helpers, and host-header parsing. Nothing here knows what a repository
//! is — that is `nr-repository`, which sits on top of this.
//!
//! The split lands here rather than one crate higher because of the orphan rule.
//! [`utils::IntoErrorResponse`] is implemented in [`error`] for a dozen foreign error types
//! (`sqlx::Error`, `reqwest::Error`, `argon2::Error` and friends), so the trait and those impls
//! have to share a crate — and that crate is then a dependency of everything that returns an
//! error from a handler, which is all of it.

pub mod authentication;
pub mod config;
pub mod error;
pub mod responses;
pub mod utils;
