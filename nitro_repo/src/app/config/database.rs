use sea_orm::ConnectOptions;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "settings")]
pub enum DatabaseConfig {
    Mysql(MysqlSettings),
    Sqlite(SqliteSettings),
    Postgres(PostgresSettings),
}
impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig::Sqlite(SqliteSettings::default())
    }
}
#[allow(clippy::from_over_into)]
impl Into<sea_orm::ConnectOptions> for DatabaseConfig {
    fn into(self) -> ConnectOptions {
        match self {
            DatabaseConfig::Mysql(mysql) => ConnectOptions::new(mysql.to_string()),
            DatabaseConfig::Sqlite(database) => ConnectOptions::new(database.to_string()),
            DatabaseConfig::Postgres(postgres) => ConnectOptions::new(postgres.to_string()),
        }
    }
}

impl Display for MysqlSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mysql://{}:{}@{}/{}",
            self.user, self.password, self.host, self.database
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MysqlSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostgresSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}
impl Display for PostgresSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "postgres://{}:{}@{}/{}",
            self.user, self.password, self.host, self.database
        )
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SqliteSettings {
    pub database_file: PathBuf,
}

impl Display for SqliteSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "sqlite:{}?mode=rwc", self.database_file.display())
    }
}
impl Default for SqliteSettings {
    fn default() -> Self {
        SqliteSettings {
            database_file: PathBuf::new().join("database.db"),
        }
    }
}
