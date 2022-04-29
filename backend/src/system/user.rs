use crate::error::internal_error::InternalError;
use crate::system::user;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub permissions: Json,
    pub created: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, DeriveIntoActiveModel)]
pub struct ModifyUser {
    pub name: String,
    pub email: String,
}
impl ActiveModelBehavior for ActiveModel {}
#[derive(Clone, Debug, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct NewUser {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub permissions: Json,

    pub created: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::auth_token::Entity")]
    AuthToken,
}

impl Related<super::auth_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthToken.def()
    }
}

pub async fn get_by_username(
    username: &str,
    connection: &DatabaseConnection,
) -> Result<Option<Model>, InternalError> {
    user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(connection)
        .await
        .map_err(|e| InternalError::DBError(e))
}
