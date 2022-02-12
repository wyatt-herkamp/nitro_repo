use async_trait::async_trait;
use log::error;
use serde::{Deserialize, Serialize};

use crate::error::internal_error::InternalError;
use crate::repository::deploy::DeployInfo;

#[async_trait]
pub trait WebhookHandler {
    type WebhookConfig;
    async fn handle(
        config: &Self::WebhookConfig,
        deploy_event: &DeployInfo,
    ) -> Result<(), InternalError>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordConfig {
    pub url: String,
}

pub struct DiscordHandler;
#[allow(clippy::redundant_pattern_matching)]
#[async_trait]
impl WebhookHandler for DiscordHandler {
    type WebhookConfig = DiscordConfig;

    async fn handle(
        config: &Self::WebhookConfig,
        _deploy_event: &DeployInfo,
    ) -> Result<(), InternalError> {
        let d_hook = webhook::client::WebhookClient::new(&config.url);
        let result = d_hook.send(|x| x.content("Deploy Happening!")).await;
        if let Err(_) = result {
            //TODO more details
            error!(
                "Unable to Call Discord Webhook {}. Error {}",
                config.url, error
            );
            return Err(InternalError::Error("Bad Webhook".to_string()));
        }
        return Ok(());
    }
}
