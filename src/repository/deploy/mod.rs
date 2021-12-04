
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use log::{debug, error};
use serde::de::Unexpected::Map;
use serde::de::value::MapDeserializer;
use serde::Deserialize;
use serde_json::Value;

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::system::models::User;
use crate::utils::get_current_time;
use crate::webhook::{DiscordConfig, DiscordHandler, WebhookHandler};

#[derive(Clone)]
pub struct DeployInfo {
    pub user: User,
    pub version: String,
    pub name: String,
    pub report_location: PathBuf,

}


pub async fn handle_post_deploy(repository: &Repository, deploy: DeployInfo) -> Result<(), InternalError> {
    for x in &repository.deploy_settings.webhooks {
        match x.handler.as_str() {
            "discord" => {
                let result = DiscordConfig::deserialize(MapDeserializer::new(x.settings.clone().into_iter()))?;
                DiscordHandler::handle(&result, &deploy).await?;
            }
            _ => {}
        }
    }
    if repository.deploy_settings.report_generation.active {
        let mut report = HashMap::<String, String>::new();

        for x in &repository.deploy_settings.report_generation.values {
            let x1 = match x.as_str() {
                "DeployerUsername" => {
                    deploy.name.clone()
                }
                "Time" => {
                    get_current_time().to_string()
                }
                "Version" => {
                    deploy.version.clone()
                }
                x => {
                    error!("Unable to find Report Value {}", x);
                    continue;
                }
            };
            report.insert(x.clone(), x1);
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .create(true)
            .open(&deploy.report_location)?;
        file.write_all(serde_json::to_string(&report).unwrap().as_bytes())?;
    }
    return Ok(());
}