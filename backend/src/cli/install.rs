use crate::settings::models::{
    Application, Database, GeneralSettings, MysqlSettings, PostgresSettings, SqliteSettings,
};
use crate::system::hash;
use crate::system::permissions::{RepositoryPermission, UserPermissions};
use crate::system::user::database::ActiveModel;
use crate::system::user::UserEntity;
use clap::{Parser, Subcommand};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectOptions, EntityTrait};

use chrono::Local;
use std::env;
use std::env::current_dir;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::exit;
use tokio::fs::create_dir_all;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct InstallCommand {
    #[clap(subcommand)]
    database_type: DatabaseTypes,
    #[clap(long)]
    admin_username: Option<String>,
    #[clap(long)]
    admin_password: Option<String>,
    #[clap(long)]
    frontend_path: PathBuf,
    #[clap(long)]
    storage_path: Option<PathBuf>,
    #[clap(long)]
    ignore_if_installed: Option<bool>,
    #[clap(long)]
    log_dir: Option<String>,
}

#[derive(Subcommand, Debug)]
enum DatabaseTypes {
    Mysql(MysqlInstall),
    Sqlite(SqliteInstall),
    Postgres(PostgresInstall),
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct SqliteInstall {
    #[clap(long)]
    database_file: String,
    #[clap(long)]
    skip_if_file_exists: Option<bool>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct MysqlInstall {
    #[clap(long)]
    pub db_user: String,
    #[clap(long)]
    pub db_password: String,
    #[clap(long)]
    pub db_host: String,
    #[clap(long)]
    pub database: String,
}
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct PostgresInstall {
    #[clap(long)]
    pub db_user: String,
    #[clap(long)]
    pub db_password: String,
    #[clap(long)]
    pub db_host: String,
    #[clap(long)]
    pub database: String,
}

pub async fn install_task(install_command: InstallCommand) {
    let config_dir = env::var("NITRO_CONFIG_DIR")
        .map(|x| PathBuf::from(x))
        .unwrap_or_else(|_| current_dir().unwrap());
    if !config_dir.exists() {
        create_dir_all(&config_dir).await.unwrap();
    }
    if config_dir.join("nitro_repo.toml").exists() {
        if install_command.ignore_if_installed.unwrap_or(true) {
            return;
        } else {
            exit(1);
        }
    }
    let (config, skip_if_exists) = match install_command.database_type {
        DatabaseTypes::Mysql(mysql) => {
            let mysql_settings = MysqlSettings {
                user: mysql.db_user,
                password: mysql.db_password,
                host: mysql.db_host,
                database: mysql.database,
            };
            (
                crate::settings::models::Database::Mysql(mysql_settings),
                false,
            )
        }
        DatabaseTypes::Sqlite(sqlite) => {
            let buf = Path::new(&sqlite.database_file).to_path_buf();
            let sqlite_settings = SqliteSettings { database_file: buf };
            (
                crate::settings::models::Database::Sqlite(sqlite_settings),
                sqlite.skip_if_file_exists.unwrap_or(false),
            )
        }
        DatabaseTypes::Postgres(data) => {
            let postgres_settings = PostgresSettings {
                user: data.db_user,
                password: data.db_password,
                host: data.db_host,
                database: data.database,
            };
            (
                crate::settings::models::Database::Postgres(postgres_settings),
                false,
            )
        }
    };
    let frontend_path = install_command
        .frontend_path
        .canonicalize()
        .expect("Failed to canonicalize path")
        .to_str()
        .unwrap()
        .to_string();
    let log_path = install_command
        .log_dir
        .unwrap_or("./logs".to_string())
        .to_string();
    let storage_location = install_command
        .storage_path
        .and_then(|v| Some(PathBuf::from(v)))
        .unwrap_or_else(|| env::current_dir().unwrap().join("storages"));

    match &config {
        #[cfg(feature = "sqlite")]
        Database::Sqlite(ref settings) => {
            if settings.database_file.exists() {
                if !skip_if_exists {
                    seed_data(
                        config.clone(),
                        install_command.admin_username,
                        install_command.admin_password,
                    )
                    .await;
                } else {
                    println!("Skipping seed data because file exists");
                }
            } else {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&settings.database_file)
                    .expect("Failed to open file");
                seed_data(
                    config.clone(),
                    install_command.admin_username,
                    install_command.admin_password,
                )
                .await;
            }
        }
        v => {
            seed_data(
                v.clone(),
                install_command.admin_username,
                install_command.admin_password,
            )
            .await;
        }
    }
    let general = GeneralSettings {
        database: config,
        application: Application {
            log: log_path,
            frontend: frontend_path,
            storage_location,
            ..Application::default()
        },
        internal: Default::default(),
        session: Default::default(),
    };
    crate::install::install_data(config_dir, general).expect("Failed to install data");
}

async fn seed_data(
    config: impl Into<ConnectOptions>,
    username: Option<String>,
    password: Option<String>,
) {
    let options: ConnectOptions = config.into();
    let mut database_conn = sea_orm::Database::connect(options)
        .await
        .expect("Failed to connect to database");
    crate::utils::run_database_setup(&mut database_conn)
        .await
        .expect("Failed to run database setup");
    let option = username
        .as_ref()
        .and_then(|v| Some(v.as_str()))
        .unwrap_or_else(|| "admin");
    let password: &str = password
        .as_ref()
        .and_then(|v| Some(v.as_str()))
        .unwrap_or("password");
    let user: ActiveModel = ActiveModel {
        id: Default::default(),
        name: Set(option.to_string()),
        username: Set(option.to_string()),

        email: Set("admin@nitro_repo.dev".to_string()),
        password: Set(hash(password).unwrap()),
        permissions: Set(UserPermissions {
            disabled: false,
            admin: true,
            user_manager: true,
            repository_manager: true,
            deployer: RepositoryPermission::default(),
            viewer: RepositoryPermission::default(),
        }),
        created: Set(Local::now().into()),
    };

    UserEntity::insert(user)
        .exec(&database_conn)
        .await
        .expect("Failed to insert user");
}
