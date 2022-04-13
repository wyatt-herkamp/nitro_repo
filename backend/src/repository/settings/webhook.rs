use std::collections::HashMap;
use std::ops::Deref;
use serde::{Serialize, Deserialize};
use serde_json::Value;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Webhook {
    pub id: String,
    pub handler: String,
    pub settings: HashMap<String, Value>,
}

impl PartialEq<Self> for Webhook {
    fn eq(&self, other: &Self) -> bool {
        other.id.eq_ignore_ascii_case(&self.id)
    }
}

impl PartialEq<String> for Webhook {
    fn eq(&self, other: &String) -> bool {
        self.id.eq(other)
    }
}

impl Default for ReportGeneration {
    fn default() -> Self {
        ReportGeneration {
            active: true,
            values: vec!["DeployerUsername".to_string(), "Time".to_string()],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DeploySettings {
    #[serde(default)]
    pub report_generation: ReportGeneration,
    #[serde(default)]
    pub webhooks: Vec<Webhook>,
}

impl DeploySettings {
    pub fn add_webhook(&mut self, webhook: Webhook) {
        for x in self.webhooks.iter_mut() {
            if x.deref().eq(&webhook) {
                //TODO update webhook properties
                return;
            }
        }
        self.webhooks.push(webhook);
    }
    pub fn remove_hook(&mut self, webhook: String) -> Option<Webhook> {
        let option = self.webhooks.iter().position(|x| x.eq(&webhook));
        if let Some(value) = option {
            Some(self.webhooks.remove(value))
        } else {
            None
        }
    }
}