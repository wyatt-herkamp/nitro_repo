use actix_web::{get, web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLockReadGuard;
use utoipa::Component;

use crate::authentication::Authentication;
use crate::helpers::unwrap_or_not_found;
use crate::repository::settings::RepositoryType;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct ProjectResponse {}
