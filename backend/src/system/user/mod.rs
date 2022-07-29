pub mod database;

use crate::system::user::database::UserSafeData;
pub use database::Entity as UserEntity;
pub use database::Model as UserModel;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QueryFilter, Statement,
};
use serde::Deserialize;

pub async fn get_by_username(
    username: &str,
    connection: &DatabaseConnection,
) -> Result<Option<UserSafeData>, DbErr> {
    UserEntity::find()
        .filter(database::Column::Username.eq(username))
        .into_model()
        .one(connection)
        .await
}
pub async fn get_by_id(
    id: i64,
    connection: &DatabaseConnection,
) -> Result<Option<UserSafeData>, DbErr> {
    UserEntity::find()
        .filter(database::Column::Id.eq(id))
        .into_model::<UserSafeData>()
        .one(connection)
        .await
}

pub async fn get_users(connection: &DatabaseConnection) -> Result<Vec<UserModel>, DbErr> {
    UserEntity::find().all(connection).await
}
