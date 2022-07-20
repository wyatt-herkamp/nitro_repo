use api::settings::models::{Database, GeneralSettings, MysqlSettings, SqliteSettings};
use api::system::permissions::UserPermissions;
use api::system::user::database::ActiveModel;
use api::system::user::UserEntity;
use api::system::{hash, user};
use api::utils::get_current_time;
use clap::{Parser, Subcommand};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectOptions, DatabaseConnection, EntityTrait};
use semver::Op;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct InstallCommand {
    #[clap(subcommand)]
    database_type: DatabaseTypes,
    #[clap(long)]
    admin_username: String,
    #[clap(long)]
    admin_password: String,
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
#[tokio::main]
async fn main() {
    let install_command = InstallCommand::parse();
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
        println!("{}", sql.to_string());
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
    let user: user::database::ActiveModel = ActiveModel {
        id: Default::default(),
        name: Set(install_command.admin_username.clone()),
        username: Set(install_command.admin_username),

        email: Set("admin@nitro_repo.dev".to_string()),
        password: Set(hash(install_command.admin_password).unwrap()),
        permissions: Set(UserPermissions {
            disabled: false,
            admin: true,
            user_manager: true,
            repository_manager: true,
            deployer: None,
            viewer: None,
        }),
        created: Set(get_current_time()),
    };

    UserEntity::insert(user)
        .exec(&database_conn)
        .await
        .expect("Failed to insert user");
    let general = GeneralSettings {
        database: config,
        application: Default::default(),
        internal: Default::default(),
        session: Default::default(),
    };
    api::install::install_data(env::current_dir().unwrap(), general)
        .expect("Failed to install data");
}
