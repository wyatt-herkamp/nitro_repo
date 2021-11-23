use serde::de::value::MapDeserializer;
use serde::Deserialize;

use crate::error::internal_error::InternalError;
use crate::repository::models::Repository;
use crate::webhook::{DiscordConfig, DiscordHandler, WebhookHandler};

#[derive(Clone)]
pub struct DeployInfo {}

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
    //TODO Generate Report
    return Ok(());
}