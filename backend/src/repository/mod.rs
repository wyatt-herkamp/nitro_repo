use std::ops::Deref;

use tokio::sync::RwLockReadGuard;

use crate::error::internal_error::InternalError;
use crate::repository::ci::CIHandler;
use crate::repository::docker::DockerHandler;
use crate::repository::handler::DynamicRepositoryHandler;
use crate::repository::maven::MavenHandler;
use crate::repository::nitro::dynamic::DynamicNitroRepositoryHandler;
use crate::repository::npm::NPMHandler;
use crate::repository::raw::RawHandler;
use crate::repository::settings::{RepositoryConfig, RepositoryType};
use crate::storage::models::Storage;

pub mod ci;
pub mod docker;
pub mod frontend;
pub mod handler;
pub mod maven;
pub mod nitro;
pub mod npm;
pub mod raw;
pub mod response;
pub mod settings;
pub mod web;

pub static REPOSITORY_CONF: &str = "repository.nitro_repo";
pub static REPOSITORY_CONF_FOLDER: &str = ".config.nitro_repo";
pub static REPOSITORY_CONF_BAK: &str = "repository.nitro_repo.bak";
pub use handler::get_repository_handler;
