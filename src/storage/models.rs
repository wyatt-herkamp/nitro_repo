use serde::{Deserialize, Serialize};

use crate::schema::*;

use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::Mysql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{deserialize, serialize, MysqlConnection};

use crate::schema::*;

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "storages"]
pub struct Storage {
    pub id: i64,
    pub public_name: String,
    pub name: String,
    pub created: i64,
}
