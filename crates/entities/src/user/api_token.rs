use sea_orm::{entity::prelude::*, FromJsonQueryResult};

use sea_orm_exports::SeaORMExports;
use serde::{Deserialize, Serialize};

use super::{UserEntity, UserSafeData};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct TokenProperties {
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, SeaORMExports, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "auth_tokens")]
#[exports(AuthToken, has_relation)]

pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub token_hash: String,
    pub properties: TokenProperties,
    pub user_id: i64,
    pub created: DateTimeWithTimeZone,
}

impl Model {
    pub async fn get_user(
        &self,
        database: &DatabaseConnection,
    ) -> Result<Option<UserSafeData>, DbErr> {
        UserEntity::find_by_id(self.user_id)
            .into_model()
            .one(database)
            .await
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Entity",
        from = "Column::UserId",
        to = "super::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
