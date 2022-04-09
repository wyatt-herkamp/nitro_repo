use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;

use badge_maker::Style;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::repository::models::Policy::Mixed;
use crate::repository::models::Visibility::Public;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositorySummary {
    pub name: String,
    pub storage: String,
    pub repo_type: String,
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

impl RepositorySummary {
    pub fn new(repo: &Repository) -> RepositorySummary {
        RepositorySummary {
            name: repo.name.clone(),
            storage: repo.storage.clone(),
            repo_type: repo.repo_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeStyle {
    Flat,
    FlatSquare,
    Plastic,
}

impl Default for BadgeStyle {
    fn default() -> Self {
        BadgeStyle::Flat
    }
}

impl BadgeStyle {
    pub fn to_badge_maker_style(&self) -> badge_maker::Style {
        match self {
            BadgeStyle::Flat => Style::Flat,
            BadgeStyle::FlatSquare => Style::FlatSquare,
            BadgeStyle::Plastic => Style::Plastic,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeSettings {
    #[serde(default = "BadgeStyle::default")]
    pub style: BadgeStyle,
    #[serde(default = "default_label_color")]
    pub label_color: String,
    #[serde(default = "default_color")]
    pub color: String,
}

impl Default for BadgeSettings {
    fn default() -> Self {
        BadgeSettings {
            style: Default::default(),
            label_color: default_label_color(),
            color: default_color(),
        }
    }
}

fn default_color() -> String {
    "#33B5E5".to_string()
}

fn default_label_color() -> String {
    "#555".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageProvider {
    None,
    README,
    ReadmeGit,
    ReadmeSent,
}

impl PageProvider {
    fn default() -> Self {
        PageProvider::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontend {
    #[serde(default = "default")]
    pub enabled: bool,
    #[serde(default = "PageProvider::default")]
    pub page_provider: PageProvider,
}

impl Default for Frontend {
    fn default() -> Self {
        Frontend {
            enabled: true,
            page_provider: PageProvider::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, strum_macros::EnumString)]
pub enum Policy {
    Release,
    Snapshot,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, strum_macros::EnumString)]
pub enum Visibility {
    Public,
    Private,
    Hidden,
}

impl Policy {
    fn default() -> Self {
        Mixed
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Public
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SecurityRules {
    ///Default true. If false only people listed in deployers can deploy
    ///List of deployers
    //TODO IMPLEMENT IN BACKEND
    #[serde(default = "Vec::new")]
    pub deployers: Vec<i64>,
    #[serde(default = "Visibility::default")]
    pub visibility: Visibility,
    ///List of readers
    /// If Empty it will ignore this method of security
    //TODO IMPLEMENT IN BACKEND
    #[serde(default = "Vec::new")]
    pub readers: Vec<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositorySettings {
    #[serde(default = "default")]
    pub active: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default = "Policy::default")]
    pub policy: Policy,
    #[serde(default = "Frontend::default")]
    pub frontend: Frontend,
    #[serde(default = "BadgeSettings::default")]
    pub badge: BadgeSettings,
}

impl Default for RepositorySettings {
    fn default() -> Self {
        RepositorySettings {
            active: true,
            description: "".to_string(),
            policy: Policy::Mixed,
            frontend: Default::default(),
            badge: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub active: bool,
    pub policy: Policy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFrontend {
    pub frontend: Frontend,
    pub badge: BadgeSettings,
}

impl RepositorySettings {
    pub fn update_general(&mut self, settings: UpdateSettings) {
        self.policy = settings.policy;
        self.active = settings.active;
    }
    pub fn update_frontend(&mut self, settings: UpdateFrontend) {
        self.frontend = settings.frontend;
        self.badge = settings.badge;
    }
}

impl SecurityRules {
    pub fn update(&mut self, security: SecurityRules) {
        self.visibility = security.visibility;
        self.deployers = security.deployers;
        self.readers = security.readers;
    }
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }
}

fn default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub repo_type: String,
    pub storage: String,
    pub settings: RepositorySettings,
    pub security: SecurityRules,
    #[serde(default)]
    pub deploy_settings: DeploySettings,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryListResponse {
    pub name: String,
    pub repo_type: String,
    pub storage: String,
}
