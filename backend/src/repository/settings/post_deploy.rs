use std::ops::Deref;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository::settings::RepositoryConfigType;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostDeployTasks {
    pub report_generation: ReportGeneration,
    pub webhooks: Vec<Webhook>,
}

impl RepositoryConfigType for PostDeployTasks {
    fn config_name() -> &'static str {
        "post_deploy.json"
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ReportValues {
    DeployerUsername,
    Time,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReportGeneration {
    pub active: bool,
    pub values: Vec<String>,
}

impl Default for ReportGeneration {
    fn default() -> Self {
        ReportGeneration {
            active: true,
            values: vec!["DeployerUsername".to_string(), "Time".to_string()],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WebhookHandlerConfig {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Webhook {
    pub id: Uuid,
    pub handler: WebhookHandlerConfig,
}

impl PartialEq<Self> for Webhook {
    fn eq(&self, other: &Self) -> bool {
        other.id.eq(&self.id)
    }
}

impl PartialEq<Uuid> for Webhook {
    fn eq(&self, other: &Uuid) -> bool {
        other.eq(&self.id)
    }
}

impl PostDeployTasks {
    /// If the Webhook by id already exists. It will replace it via std::mem::replace
    pub fn add_webhook(&mut self, webhook: Webhook) -> Option<Webhook> {
        for web in self.webhooks.iter_mut() {
            if web.deref().eq(&webhook) {
                return Some(std::mem::replace(web, webhook));
            }
        }
        self.webhooks.push(webhook);
        None
    }
    pub fn remove_hook(&mut self, webhook: Uuid) -> Option<Webhook> {
        let option = self.webhooks.iter().position(|x| x.eq(&webhook));
        if let Some(value) = option {
            Some(self.webhooks.remove(value))
        } else {
            None
        }
    }
}
