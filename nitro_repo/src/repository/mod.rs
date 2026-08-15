//! The repository contract and the four types that implement it.
//!
//! Nothing here any more: `nr-repository` holds the contract, and each type is its own crate. The
//! re-exports keep the existing `crate::repository::` paths resolving; they go away in the next
//! commit, once every call site names the crate it means.

pub use nr_cargo as cargo;
pub use nr_docker as docker;
pub use nr_maven as maven;
pub use nr_npm as npm;
pub use nr_repository::*;
