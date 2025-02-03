use serde::{Deserialize, Serialize};
use sqlx::postgres::PgConnectOptions;

use super::DBError;

/// The configuration for the database.
///
/// Currently only supports PostgreSQL.
#[derive(Debug, Clone, Deserialize, Serialize, clap::Args)]
#[serde(default)]
pub struct DatabaseConfig {
    /// The username to connect to the database.
    ///
    /// Default is `postgres`.
    /// Environment_variable: NITRO-REPO_DATABASE_USER
    #[clap(long = "database-user", default_value = "postgres")]
    pub user: String,
    /// The password to connect to the database.
    ///
    /// Default is `password`.
    /// Environment_variable: NITRO-REPO_DATABASE_PASSWORD
    #[clap(long = "database-password", default_value = "password")]
    pub password: String,
    #[clap(long = "database-name", default_value = "nitro_repo")]
    #[serde(alias = "name")]
    pub database: String,
    // The host can be in the format host:port or just host.
    #[clap(long = "database-host", default_value = "localhost:5432")]
    pub host: String,
    // The port is optional. If not specified the default port is used. or will be extracted from the host.
    #[clap(long = "database-port")]
    pub port: Option<u16>,
}
impl DatabaseConfig {
    /// Returns the host and port
    ///
    /// If it is not specified in the port field it will attempt to extract it from the host field.
    pub fn host_name_port(&self) -> Result<(&str, u16), DBError> {
        if let Some(port) = self.port {
            Ok((self.host.as_str(), port))
        } else {
            // The port can be specified in the host field. If it is, we need to extract it.
            let host = self.host.split(':').collect::<Vec<&str>>();

            match host.len() {
                // The port is not specified. Use the default port.
                1 => Ok((host[0], 5432)),
                // The port is specified within the host. The port option is ignored.
                2 => Ok((host[0], host[1].parse::<u16>().unwrap_or(5432))),
                _ => {
                    // Not in the format host:port. Possibly IPv6 but we don't support that.
                    // If it is IPv6 please specify the port separately.
                    Err(DBError::InvalidHost(self.host.clone()))
                }
            }
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            user: "postgres".to_string(),
            password: "password".to_string(),
            database: "nitro_repo".to_string(),
            host: "localhost".to_string(),
            port: Some(5432),
        }
    }
}
impl TryFrom<DatabaseConfig> for PgConnectOptions {
    type Error = DBError;
    fn try_from(settings: DatabaseConfig) -> Result<PgConnectOptions, Self::Error> {
        let (host, port) = settings.host_name_port()?;
        let options = PgConnectOptions::new()
            .username(&settings.user)
            .password(&settings.password)
            .host(host)
            .port(port)
            .database(&settings.database);

        Ok(options)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_host_name_port() {
        {
            let config = DatabaseConfig::default();
            let (host, port) = config.host_name_port().unwrap();
            assert_eq!(host, "localhost");
            assert_eq!(port, 5432);
        }
        {
            let config = DatabaseConfig {
                host: "localhost:5433".to_string(),
                port: None,
                ..DatabaseConfig::default()
            };
            let (host, port) = config.host_name_port().unwrap();
            assert_eq!(host, "localhost");
            assert_eq!(port, 5433);
        }
    }
}
