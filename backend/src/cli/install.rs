use api::settings::models::{Application, Database, GeneralSettings, MysqlSettings, SqliteSettings};
use api::system::permissions::{RepositoryPermission, UserPermissions};
use api::system::user::database::ActiveModel;
use api::system::user::UserEntity;
use api::system::{hash, user};
use api::utils::get_current_time;
use clap::{Parser, Subcommand};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectOptions, EntityTrait};

use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::exit;
use crate::GeneralSettings;
use crate::settings::models::{Application, Database, MysqlSettings, SqliteSettings};
use crate::system::{hash, user};
use crate::system::permissions::{RepositoryPermission, UserPermissions};
use crate::system::user::database::ActiveModel;
use crate::system::user::UserEntity;
use crate::utils::get_current_time;

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
    let config = match install_command.database_type {
        DatabaseTypes::Mysql(mysql) => {
            let mysql_settings = MysqlSettings {
                user: mysql.db_user,
                password: mysql.db_password,
                host: mysql.db_host,
                database: mysql.database,
            };
            api::settings::models::Database::Mysql(mysql_settings)
        }
        DatabaseTypes::Sqlite(sqlite) => {
            let buf = Path::new(&sqlite.database_file).to_path_buf();
            let sqlite_settings = SqliteSettings { database_file: buf };
            api::settings::models::Database::Sqlite(sqlite_settings)
        }
    };
    if let Database::Sqlite(sql) = &config {
        println!("{}", sql);
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&sql.database_file)
            .expect("Failed to open file");
    }
    let options: ConnectOptions = config.clone().into();
    let mut database_conn = sea_orm::Database::connect(options)
        .await
        .expect("Failed to connect to database");
    api::utils::run_database_setup(&mut database_conn)
        .await
        .expect("Failed to run database setup");
    let option = install_command.admin_username.unwrap_or_else(|| whoami::username());
    let password: &str = install_command.admin_password.as_ref().and_then(|v| Some(v.as_str())).unwrap_or("password");
    let user: ActiveModel = ActiveModel {
        id: Default::default(),
        name: Set(option.clone()),
        username: Set(option),

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
    let general = GeneralSettings {
        database: config,
        application: Application {
            log: install_command.log_dir.unwrap_or("./logs".to_string()).to_string(),
            frontend: install_command.frontend_path,
            storage_location: install_command.storage_path.and_then(|v| Some(PathBuf::from(v))).unwrap_or_else(|| {
                env::current_dir().unwrap().join("storages")
            }),
            ..Application::default()
        },
        internal: Default::default(),
        session: Default::default(),
    };
    api::install::install_data(working_directory, general)
        .expect("Failed to install data");
}
