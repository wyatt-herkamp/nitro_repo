use ahash::{HashMap, HashMapExt};

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    database::DateTime,
    user::permissions::{RepositoryActions, UserPermissions},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, ToSchema, FromRow)]

pub struct UserRepositoryPermissions {
    pub id: i32,
    pub user_id: i32,
    pub repository_id: Uuid,
    pub actions: Vec<RepositoryActions>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}
impl UserRepositoryPermissions {
    pub async fn has_repository_action(
        user_id: i32,
        repository: Uuid,
        action: RepositoryActions,
        database: &PgPool,
    ) -> sqlx::Result<bool> {
        let Some(actions) = sqlx::query_scalar::<_, Vec<RepositoryActions>>(
            r#"SELECT * FROM user_repository_permissions WHERE user_id = $1 AND repository_id = $2 "#,
        )
        .bind(user_id)
        .bind(repository)
        .fetch_optional(database)
        .await? else{
            return Ok(false);
        };
        Ok(actions.contains(&action))
    }
    pub async fn get_all_for_user_as_map(
        user_id: i32,
        database: &PgPool,
    ) -> sqlx::Result<HashMap<Uuid, Vec<RepositoryActions>>> {
        let permissions = sqlx::query_scalar::<_, (Uuid, Vec<RepositoryActions>)>(
            r#"SELECT (repository_id, actions) FROM user_repository_permissions WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_all(database)
        .await?;
        let mut map = HashMap::new();
        for (repository, actions) in permissions {
            map.insert(repository, actions);
        }
        Ok(map)
    }
    pub async fn delete(user_id: i32, repository_id: Uuid, database: &PgPool) -> sqlx::Result<()> {
        sqlx::query(
            r#"DELETE FROM user_repository_permissions WHERE user_id = $1 AND repository_id = $2"#,
        )
        .bind(user_id)
        .bind(repository_id)
        .execute(database)
        .await?;
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, ToSchema, FromRow)]

pub struct NewUserRepositoryPermissions {
    pub user_id: i32,
    pub repository_id: Uuid,
    pub actions: Vec<RepositoryActions>,
}
impl NewUserRepositoryPermissions {
    #[instrument]
    pub async fn insert(self, database: &PgPool) -> sqlx::Result<i32> {
        let row:i32 = sqlx::query_scalar(
            r#"INSERT INTO user_repository_permissions (user_id, repository_id, actions) VALUES ($1, $2, $3)
                        ON CONFLICT (user_id, repository_id) DO UPDATE SET actions = $3
                    RETURNING id"#,
        )
        .bind(self.user_id)
        .bind(self.repository_id)
        .bind(self.actions)
        .fetch_one(database)
        .await?;
        Ok(row)
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, ToSchema)]
pub struct FullUserPermissions {
    pub user_id: i32,
    pub admin: bool,
    pub user_manager: bool,
    /// Repository Manager will be able to create and delete repositories
    /// Also will have full read/write access to all repositories
    pub system_manager: bool,
    pub default_repository_actions: Vec<RepositoryActions>,
    pub repository_permissions: HashMap<Uuid, Vec<RepositoryActions>>,
}
impl FullUserPermissions {
    pub async fn get_by_id(user_id: i32, database: &PgPool) -> sqlx::Result<Option<Self>> {
        let permissions =
            sqlx::query_as::<_, UserPermissions>(r#"SELECT * FROM users WHERE id = $1"#)
                .bind(user_id)
                .fetch_optional(database)
                .await?;
        let Some(permissions) = permissions else {
            return Ok(None);
        };
        let repository_permissions =
            UserRepositoryPermissions::get_all_for_user_as_map(user_id, database).await?;
        let permissions = FullUserPermissions {
            user_id,
            admin: permissions.admin,
            user_manager: permissions.user_manager,
            system_manager: permissions.system_manager,
            default_repository_actions: permissions.default_repository_actions,
            repository_permissions,
        };
        Ok(Some(permissions))
    }
}
