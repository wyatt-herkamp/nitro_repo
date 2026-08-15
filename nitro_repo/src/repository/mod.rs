//! The four repository types, and a re-export of the contract they implement.
//!
//! The contract itself — [`Repository`], [`RepositoryType`], [`RepoResponse`], [`SiteContext`] and
//! the HTTP plumbing around them — lives in `nr-repository`. It is re-exported here so that the
//! existing `crate::repository::` paths keep resolving while the four types below are still in
//! this crate; the glob goes away once each of them is its own crate too.

pub use nr_repository::*;

pub mod docker;
pub mod npm;

/// Their own crates now. Aliased so `crate::repository::{maven,cargo}::` still resolve.
pub use {nr_cargo as cargo, nr_maven as maven};
