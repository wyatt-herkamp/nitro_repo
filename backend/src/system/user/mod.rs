pub mod database;

use crate::error::internal_error::InternalError;
pub use database::Entity as UserEntity;
pub use database::Model as UserModel;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn get_by_username(
    username: &str,
    connection: &DatabaseConnection,
) -> Result<Option<UserModel>, InternalError> {
    UserEntity::find()
        .filter(database::Column::Username.eq(username))
        .one(connection)
        .await
        .map_err(InternalError::DBError)
}
