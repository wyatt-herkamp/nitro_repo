use crate::error::internal_error::InternalError;
use crate::system::user;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::system::permissions::UserPermissions;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[sea_orm(column_type = "Json")]
    pub permissions: UserPermissions,
    pub created: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, DeriveIntoActiveModel)]
pub struct ModifyUser {
    pub name: String,
    pub email: String,
}

impl ActiveModelBehavior for ActiveModel {}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::authentication::auth_token::database::Entity")]
    AuthToken,
}

impl Related<crate::authentication::auth_token::database::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthToken.def()
    }
}

