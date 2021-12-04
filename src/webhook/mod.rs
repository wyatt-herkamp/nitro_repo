use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::deploy::DeployInfo;
#[async_trait]
pub trait WebhookHandler {
    type WebhookConfig;
    async fn handle(config: &Self::WebhookConfig, deploy_event: &DeployInfo) -> Result<(), InternalError>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordConfig {
    pub url: String,
}

pub struct DiscordHandler;

#[async_trait]
impl WebhookHandler for DiscordHandler {
    type WebhookConfig = DiscordConfig;

    async fn handle(config: &Self::WebhookConfig, deploy_event: &DeployInfo) -> Result<(), InternalError> {
        let d_hook = webhook::Webhook::from_url(&config.url);
        d_hook.send(|x| {
            x.content("Deploy Happening!")
        }).await?;
        return Ok(());
    }
}