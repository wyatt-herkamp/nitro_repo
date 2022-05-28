pub mod database;

pub use database::Entity as UserEntity;
pub use database::Model as UserModel;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub async fn get_by_username(
    username: &str,
    connection: &DatabaseConnection,
) -> Result<Option<UserModel>, DbErr> {
    UserEntity::find()
        .filter(database::Column::Username.eq(username))
        .one(connection)
        .await
}
