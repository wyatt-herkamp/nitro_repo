#![allow(clippy::from_over_into)]
#![allow(irrefutable_let_patterns)]
#![deny(deprecated)]
// A lot of macros add catch all patterns
#![allow(unreachable_patterns)]

use actix_web::web::Data;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::settings::models::{GeneralSettings, Settings};

pub mod authentication;
pub mod constants;
pub mod error;
pub mod frontend;
pub mod generators;
pub mod helpers;
pub mod install;
pub mod repository;
pub mod settings;
pub mod storage;
pub mod system;
pub mod utils;
#[cfg(feature = "clap")]
pub mod cli;

#[derive(Debug)]
pub struct NitroRepo {
    pub settings: RwLock<Settings>,
    pub core: GeneralSettings,
    pub current_version: Version,
}

#[derive(Serialize, Debug, Clone)]
pub struct Version {
    /// What the local config says the version is
    pub installed: semver::Version,
    /// The Cargo Build Version
    pub cargo_version: semver::Version,
    pub git_branch: &'static str,
    pub git_commit: &'static str,
    /// The channel rust is in
    pub mode: &'static str,
    /// Build Timestamp
    pub build_timestamp: &'static str,
    /// Rust Version
    pub rust_version: &'static str,
    /// Features enabled at compile time
    pub features: Vec<&'static str>,
}

impl Version {
    pub fn new(installed: semver::Version) -> Version {
        Version {
            installed,
            cargo_version: semver::Version::parse(env!("VERGEN_BUILD_SEMVER")).unwrap(),
            git_branch: env!("VERGEN_GIT_BRANCH"),
            git_commit: env!("VERGEN_GIT_SHA"),
            mode: env!("VERGEN_RUSTC_CHANNEL"),
            build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),
            rust_version: env!("VERGEN_RUSTC_SEMVER"),
            features: env!("VERGEN_CARGO_FEATURES").split(",").collect(),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Version::new(semver::Version::parse(env!("VERGEN_BUILD_SEMVER")).unwrap())
    }
}

pub type NitroRepoData = Data<NitroRepo>;
