use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::error::internal_error::InternalError;
use crate::system::auth_token;

#[derive(Clone, Debug, PartialEq,DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "auth_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub token: String,
    pub expiration: i64,
    pub created: i64,
    pub user_id: Option<i64>,

}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::user::Entity",
    from = "Column::UserId",
    to = "super::user::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
pub async fn get_by_token(token: &str, connection: &DatabaseConnection) ->Result<Option<Model>, InternalError>{
    auth_token::Entity::find().filter(auth_token::Column::Token.eq(token)).one(connection).await.map_err(|e|InternalError::DBError(e))

}