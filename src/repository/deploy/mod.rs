use log::error;
use std::collections::HashMap;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;

use serde::de::value::MapDeserializer;
use serde::Deserialize;

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

pub async fn handle_post_deploy(
    repository: &Repository,
    deploy: DeployInfo,
) -> Result<(), InternalError> {
    for x in &repository.deploy_settings.webhooks {
        match x.handler.as_str() {
            "discord" => {
                let result = DiscordConfig::deserialize(MapDeserializer::new(
                    x.settings.clone().into_iter(),
                ))?;
                DiscordHandler::handle(&result, &deploy).await?;
            }
            _ => {}
        }
    }
    if repository.deploy_settings.report_generation.active {
        let mut report = HashMap::<String, String>::new();

        for x in &repository.deploy_settings.report_generation.values {
            let x1 = match x.as_str() {
                "DeployerUsername" => deploy.user.name.clone(),
                "Time" => get_current_time().to_string(),
                "Version" => deploy.version.clone(),
                x => {
                    error!("Unable to find Report Value {}", x);
                    continue;
                }
            };
            report.insert(x.clone(), x1);
        }
        let buf = deploy.report_location;
        if buf.exists() {
            remove_file(&buf)?;
        }
        let mut file = File::create(buf)?;
        let string = serde_json::to_string(&report).unwrap();
        let x1 = string.as_bytes();
        file.write_all(x1)?;
    }
    return Ok(());
}
