use actix_web::web::Data;
use config::SiteSetting;
use nr_core::database::user::does_user_exist;
use parking_lot::Mutex;
use serde::Serialize;

pub mod authentication;
pub mod config;
pub mod email;
pub mod logging;
use current_semver::current_semver;
use sqlx::PgPool;
use tracing::{info, instrument};
pub mod api;
pub mod web;
#[derive(Debug, Serialize, Clone)]
pub struct Instance {
    pub app_url: String,
    pub name: String,
    pub description: String,
    pub is_https: bool,
    pub is_installed: bool,
    pub version: semver::Version,
}

#[derive(Debug)]
pub struct NitroRepo {
    pub instance: Mutex<Instance>,
}

impl NitroRepo {
    pub async fn new(site: SiteSetting, database: DatabaseConnection) -> anyhow::Result<Self> {
        let is_installed = does_user_exist(&database).await?;
        let instance = Instance {
            version: current_semver!(),
            app_url: site.app_url.unwrap_or_default(),
            is_installed,
            name: site.name,
            description: site.description,
            is_https: site.is_https,
        };
        Ok(NitroRepo {
            instance: Mutex::new(instance),
        })
    }
    #[instrument]
    pub fn update_app_url(&self, app_url: String) {
        let mut instance = self.instance.lock();
        if instance.app_url.is_empty() {
            info!("Updating app url to {}", app_url);
            instance.app_url = app_url;
        }
    }
}

pub type DatabaseConnection = Data<PgPool>;
pub type NitroRepoData = Data<NitroRepo>;
