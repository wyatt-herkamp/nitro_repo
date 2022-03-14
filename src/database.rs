use actix_web::web;
use diesel::{Connection, MysqlConnection};
use diesel::r2d2::ConnectionManager;
use diesel_migrations::embed_migrations;
use anyhow::Result;
use log::info;

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub type Database = web::Data<DbPool>;
embed_migrations!();

pub fn init(db_url: &str) -> Result<DbPool> {
    info!("Loading Database");
    let manager = ConnectionManager::<MysqlConnection>::new(db_url);
    let pool = DbPool::new(manager)?;
    let conn = pool.get()?;
    info!("Checking and Running Migrations");
    embedded_migrations::run(&conn)?;
    Ok(pool)
}

pub fn init_single_connection(db_url: &str) -> Result<MysqlConnection> {
    info!("Loading Database");
    let connection = MysqlConnection::establish(db_url)?;

    info!("Checking and Running Migrations");
    embedded_migrations::run(&connection)?;
    Ok(connection)
}