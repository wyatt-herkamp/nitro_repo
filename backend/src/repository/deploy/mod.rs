use std::fmt::{Display, Formatter};

use serde::de::value::MapDeserializer;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::storage::models::StringStorage;
use crate::system::user::UserModel;

use crate::webhook::{DiscordConfig, DiscordHandler, WebhookHandler};

#[derive(Clone, Deserialize, Serialize)]
pub struct DeployInfo {
    pub user: UserModel,
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
    _storage: &StringStorage,
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
    Ok(())
}
