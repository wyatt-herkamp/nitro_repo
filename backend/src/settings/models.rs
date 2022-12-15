use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::create_dir_all;
use std::path::PathBuf;

use sea_orm::ConnectOptions;
use semver::Version;
use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mode {
    Debug,
    Release,
    Install,
}

impl AsRef<Mode> for Mode {
    fn as_ref(&self) -> &Mode {
        self
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub email: EmailSetting,
    pub site: SiteSetting,
    pub security: SecuritySettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Internal {
    pub installed: bool,
    pub version: Version,
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            installed: true,
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "settings")]
pub enum Database {
    #[cfg(feature="mysql")]
    Mysql(MysqlSettings),
    #[cfg(feature="sqlite")]
    Sqlite(SqliteSettings),
    #[cfg(feature="postgres")]
    Postgres(PostgresSettings),
}

#[allow(clippy::from_over_into)]
impl Into<sea_orm::ConnectOptions> for Database {
    fn into(self) -> ConnectOptions {
        match self {
            #[cfg(feature="mysql")]
            Database::Mysql(mysql) => ConnectOptions::new(mysql.to_string()),
            #[cfg(feature="sqlite")]
            Database::Sqlite(database) => ConnectOptions::new(database.to_string()),
            #[cfg(feature="postgres")]
            Database::Postgres(postgres) => {
                ConnectOptions::new(postgres.to_string())
            }

        }
    }
}

pub type StringMap = HashMap<String, String>;

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
        write!(f, "sqlite:{}", self.database_file.display())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Application {
    pub log: String,
    pub frontend: String,
    pub address: String,
    pub app_url: Option<String>,
    pub max_upload: usize,
    pub mode: Mode,
    pub storage_location: PathBuf,
    pub ssl_private_key: Option<String>,
    pub ssl_cert_key: Option<String>,
}

impl Default for Application {
    fn default() -> Self {
        let buf = PathBuf::from("storages");
        create_dir_all(&buf).unwrap();
        Self {
            log: "./".to_string(),
            frontend: "frontend".to_string(),
            address: "0.0.0.0:6742".to_string(),
            app_url: None,
            max_upload: 1024,
            mode: Mode::Release,
            storage_location: buf.canonicalize().unwrap(),
            ssl_private_key: None,
            ssl_cert_key: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralSettings {
    pub database: Database,
    pub application: Application,
    pub internal: Internal,
    #[serde(default)]
    pub session: SessionSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SiteSetting {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionSettings {
    pub manager: String,
    pub value: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecuritySettings {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailSetting {
    pub username: String,
    pub password: String,
    pub host: String,
    pub encryption: String,
    pub from: String,
    pub port: u16,
}

#[allow(clippy::derivable_impls)]
impl Default for SecuritySettings {
    fn default() -> Self {
        SecuritySettings {}
    }
}

impl Default for SiteSetting {
    fn default() -> Self {
        SiteSetting {
            name: "nitro_repo".to_string(),
            description: "nitro_repo".to_string(),
        }
    }
}

impl Default for EmailSetting {
    fn default() -> Self {
        EmailSetting {
            username: "no-reply@example.com".to_string(),
            password: "".to_string(),
            host: "example.com".to_string(),
            encryption: "TLS".to_string(),
            from: "no-reply@example.com".to_string(),
            port: 587,
        }
    }
}

impl Default for SessionSettings {
    fn default() -> Self {
        SessionSettings {
            manager: "BasicSessionManager".to_string(),
            value: None,
        }
    }
}
