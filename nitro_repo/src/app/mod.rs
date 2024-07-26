use actix_web::web::Data;
use config::SiteSetting;
use futures::lock::Mutex;
use serde::Serialize;

pub mod authentication;
pub mod config;
pub mod email;
pub mod logging;
use current_semver::current_semver;
pub mod web;
#[derive(Debug, Serialize)]
pub struct Instance {
    pub app_url: String,
    pub name: String,
    pub description: String,
    pub is_https: bool,
    pub version: semver::Version,
}

#[derive(Debug)]
pub struct NitroRepo {
    pub instance: Mutex<Instance>,
}

impl NitroRepo {
    pub async fn new(site: SiteSetting, database: DataDatabase) -> anyhow::Result<Self> {
        let instance = Instance {
            version: current_semver!(),
            app_url: site.app_url.unwrap_or_default(),
            name: site.name,
            description: site.description,
            is_https: site.is_https,
        };
        Ok(NitroRepo {
            instance: Mutex::new(instance),
        })
    }
}

pub type DataDatabase = Data<sea_orm::DatabaseConnection>;

pub type NitroRepoData = Data<NitroRepo>;
