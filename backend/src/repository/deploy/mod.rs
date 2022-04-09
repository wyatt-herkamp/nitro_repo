use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::constants::VERSION_DATA;
use log::error;
use serde::de::value::MapDeserializer;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::storage::models::StringStorage;
use crate::system::models::User;
use crate::utils::get_current_time;
use crate::webhook::{DiscordConfig, DiscordHandler, WebhookHandler};

#[derive(Clone, Deserialize, Serialize)]
pub struct DeployInfo {
    pub user: User,
    pub version: String,
    pub name: String,
    pub version_folder: String,
}

impl Display for DeployInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub async fn handle_post_deploy(
    storage: &StringStorage,
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
    let mut report = HashMap::<&str, String>::new();
    report.insert("Version", deploy.version.clone());
    if repository.deploy_settings.report_generation.active {
        for x in &repository.deploy_settings.report_generation.values {
            let x1 = match x.as_str() {
                "DeployerUsername" => deploy.user.name.clone(),
                "Time" => get_current_time().to_string(),
                x => {
                    error!("Unable to find Report Value {}", x);
                    continue;
                }
            };
            report.insert(x, x1);
        }
    }
    let string = serde_json::to_string(&report).unwrap();
    storage.save_file(
        repository,
        string.as_bytes(),
        &format!("{}/{}", deploy.version_folder, VERSION_DATA),
    )?;
    Ok(())
}
