use std::str::FromStr;

use env_file::EnvFile;
use sqlx::PgPool;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
pub mod env_file;
pub mod logging;
use crate::{
    database::{
        user::{NewUserRequest, UserSafeData, UserType},
        DateTime,
    },
    user::{Email, Username},
};
/// The password for the test user
pub static TEST_USER_USERNAME: &str = "test_user";

pub static TEST_USER_PASSWORD: &str = "password";
static TEST_USER_PASSWORD_HASHED: &str =
    "$argon2id$v=19$m=16,t=2,p=1$b1o5VWFvVFYxRTFhUUJjeA$bpK+ySI4DIDIOh4emBFTqw";
/// Table Name: `nr_test_environment`
static TEST_INFO_TABLE: &str = include_str!("test_info.sql");
static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

pub struct TestCore {
    pub db: PgPool,
}
impl TestCore {
    pub async fn new(function_path: String) -> anyhow::Result<(Self, TestInfoEntry)> {
        let env_file = env_file::EnvFile::load("nr_tests.env")?;
        Self::start_logger(&env_file);
        let database = Self::connect(&env_file).await?;
        let new = Self { db: database };
        new.init_test_environment().await?;

        let entry = TestInfoEntry::get_or_create(&function_path, &new.db).await?;
        Ok((new, entry))
    }
    fn start_logger(env_file: &EnvFile) {
        let log = env_file.get("LOG");
        if let Some(log) = log {
            let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log.into());

            LOGGING_INIT.call_once(|| {
                let stdout_log = tracing_subscriber::fmt::layer().pretty();
                match tracing_subscriber::registry()
                    .with(stdout_log.with_filter(env_filter))
                    .try_init()
                {
                    Ok(_) => {
                        debug!("Logging initialized");
                    }
                    Err(err) => {
                        eprintln!("Error initializing logging: {}", err);
                    }
                };
            });
        }
    }
    async fn connect(env_file: &EnvFile) -> anyhow::Result<PgPool> {
        let env = env_file.get("DATABASE_URL").unwrap();
        debug!("Connecting to database {}", env);
        let db = PgPool::connect(&env).await?;
        Ok(db)
    }
    async fn init_test_environment(&self) -> anyhow::Result<()> {
        crate::database::migration::run_migrations(&self.db).await?;
        sqlx::query(TEST_INFO_TABLE).execute(&self.db).await?;
        Ok(())
    }

    pub async fn get_test_user(&self) -> anyhow::Result<Option<UserSafeData>> {
        if let Some(user) = UserSafeData::get_by_id(1, &self.db).await? {
            Ok(Some(user))
        } else {
            let user = NewUserRequest {
                name: "Test User".to_string(),
                username: Username::from_str(TEST_USER_USERNAME)?,
                email: Email::from_str("testing@example.com")?,
                password: Some(TEST_USER_PASSWORD_HASHED.to_owned()),
            };
            let user = user.insert_admin(&self.db).await?;
            Ok(Some(user.into()))
        }
    }
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TestInfoEntry {
    pub id: i32,
    pub function_path: String,
    pub run_successfully: Option<bool>,
    pub started_at: Option<DateTime>,
}
impl TestInfoEntry {
    pub async fn get_or_create(
        function_path: &str,
        db: &PgPool,
    ) -> Result<TestInfoEntry, sqlx::Error> {
        let entry = sqlx::query_as::<_, TestInfoEntry>(
            r#"INSERT INTO nr_test_environment (function_path) VALUES($1) ON CONFLICT (function_path) DO UPDATE
            SET run_successfully = null, started_at = CURRENT_TIMESTAMP
            RETURNING *;"#,
        )
        .bind(function_path.to_owned())
        .fetch_one(db)
        .await?;
        Ok(entry)
    }

    pub async fn set_success(&self, db: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(r#"UPDATE nr_test_environment SET run_successfully = true WHERE id = $1;"#)
            .bind(self.id)
            .execute(db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[ignore = "Requires a database"]
    #[tokio::test]
    pub async fn test_test_core() {
        let (core, entry) = super::TestCore::new(format!("{}::test_test_core", module_path!()))
            .await
            .unwrap();
        let user = core.get_test_user().await.unwrap();
        assert!(user.is_some());
        entry.set_success(&core.db).await.unwrap();
    }
}
