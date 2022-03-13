use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{File, remove_file};
use std::io::Write;
use std::path::PathBuf;

use log::error;
use serde::{Deserialize, Serialize};
use serde::de::value::MapDeserializer;


use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::system::models::User;
use crate::utils::get_current_time;
use crate::webhook::{DiscordConfig, DiscordHandler, WebhookHandler};

#[derive(Clone, Deserialize, Serialize)]
pub struct DeployInfo {
    pub user: User,
    pub version: String,
    pub name: String,
    pub report_location: PathBuf,
}

impl Display for DeployInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub async fn handle_post_deploy(
    repository: &Repository,
    deploy: &DeployInfo,
) -> Result<(), InternalError> {
    for x in &repository.deploy_settings.webhooks {
        if x.handler.as_str() == "discord" {
            let result =
                DiscordConfig::deserialize(MapDeserializer::new(x.settings.clone().into_iter()))?;
            DiscordHandler::handle(&result, deploy).await?;
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
        if deploy.report_location.exists() {
            remove_file(&deploy.report_location)?;
        }
        let mut file = File::create(&deploy.report_location)?;
        let string = serde_json::to_string(&report).unwrap();
        let x1 = string.as_bytes();
        file.write_all(x1)?;
    }
    return Ok(());
}
