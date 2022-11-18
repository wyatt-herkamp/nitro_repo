use crate::settings::models::{
    Application, Database, GeneralSettings, MysqlSettings, SqliteSettings,
};
use crate::system::hash;
use crate::system::permissions::{RepositoryPermission, UserPermissions};
use crate::system::user::database::ActiveModel;
use crate::system::user::UserEntity;
use crate::utils::get_current_time;
use clap::{Parser, Subcommand};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectOptions, EntityTrait};

use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::exit;

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
    frontend_path: String,
    #[clap(long)]
    storage_path: Option<String>,
    #[clap(long)]
    ignore_if_installed: Option<bool>,
    #[clap(long)]
    log_dir: Option<String>,
}

#[derive(Subcommand, Debug)]
enum DatabaseTypes {
    Mysql(MysqlInstall),
    Sqlite(SqliteInstall),
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

pub async fn install_task(install_command: InstallCommand) {
    let working_directory = env::current_dir().unwrap();
    if working_directory.join("nitro_repo.toml").exists() {
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
    };
    match &config {
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
        Database::Mysql(ref v) => {
            seed_data(
                config.clone(),
                install_command.admin_username,
                install_command.admin_password,
            )
            .await;
        }
    }

    let general = GeneralSettings {
        database: config,
        application: Application {
            log: install_command
                .log_dir
                .unwrap_or("./logs".to_string())
                .to_string(),
            frontend: install_command.frontend_path,
            storage_location: install_command
                .storage_path
                .and_then(|v| Some(PathBuf::from(v)))
                .unwrap_or_else(|| env::current_dir().unwrap().join("storages")),
            ..Application::default()
        },
        internal: Default::default(),
        session: Default::default(),
    };
    crate::install::install_data(working_directory, general).expect("Failed to install data");
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
    let password: &str = username
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
        created: Set(get_current_time()),
    };

    UserEntity::insert(user)
        .exec(&database_conn)
        .await
        .expect("Failed to insert user");
}
