use derive_more::derive::From;
use nr_macros::Scopes;
use serde::Serialize;
use sqlx::prelude::Type;
use strum::EnumIter;
use thiserror::Error;
use utoipa::ToSchema;
#[derive(Debug, Error, From)]
#[error("Invalid Scope: {0}")]
pub struct InvalidScope(pub String);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Type, ToSchema, EnumIter, Scopes)]
#[sqlx(type_name = "TEXT")]
pub enum NRScope {
    /// Can read all repositories the user has access to
    #[scope(title = "Read Repository", parent = "Repository")]
    ReadRepository,
    /// Can write to all repositories the user has access to
    #[scope(title = "Write Repository", parent = "Repository")]
    WriteRepository,
    /// Can edit all repositories the user has access to
    #[scope(title = "Edit Repository", parent = "Repository")]
    EditRepository,
    #[scope(title = "Update Repository", parent = "User")]
    /// Update your password
    UpdatePassword,
}
#[derive(Debug, Serialize, PartialEq, Eq, Hash, ToSchema)]
pub struct ScopeDescription {
    pub key: NRScope,
    pub description: &'static str,
    pub name: &'static str,
    pub parent: Option<&'static str>,
    pub requires_user_manager: bool,
    pub requires_admin: bool,
    pub requires_system: bool,
}

impl Default for ScopeDescription {
    fn default() -> Self {
        Self {
            key: NRScope::ReadRepository,
            description: "",
            name: "",
            parent: None,
            requires_user_manager: false,
            requires_admin: false,
            requires_system: false,
        }
    }
}
