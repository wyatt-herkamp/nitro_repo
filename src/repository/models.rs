use serde::{Deserialize, Serialize};

use crate::schema::*;

use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::Mysql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{deserialize, serialize, MysqlConnection};

use crate::schema::*;

use crate::repository::models::Policy::Mixed;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Policy {
    Release,
    Snapshot,
    Mixed,
}
impl Policy {
    fn default() -> Self {
        return Mixed;
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRules {
    //Default true. If false only people listed in deployers can deploy
    pub open_to_all_deployers: bool,
    //List of deployers
    pub deployers: Vec<i64>,
    #[serde(default = "default")]
    pub public: bool,
}

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow, Clone)]
#[sql_type = "Text"]
pub struct RepositorySettings {
    pub security_rules: Option<SecurityRules>,
    #[serde(default = "default")]
    pub active: bool,
    #[serde(default = "default")]
    pub re_deployment: bool,
    #[serde(default = "Policy::default")]
    pub policy: Policy,
}
fn default() -> bool {
    true
}

impl FromSql<Text, Mysql> for RepositorySettings {
    fn from_sql(
        bytes: Option<&<diesel::mysql::Mysql as Backend>::RawValue>,
    ) -> deserialize::Result<RepositorySettings> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        let result: RepositorySettings = serde_json::from_str(t.as_str())?;
        return Ok(result);
    }
}

impl ToSql<Text, Mysql> for RepositorySettings {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Mysql>) -> serialize::Result {
        let s = serde_json::to_string(&self)?;
        <String as ToSql<Text, Mysql>>::to_sql(&s, out)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "repositories"]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub repo_type: String,
    pub storage: i64,
    pub settings: RepositorySettings,
    pub created: i64,
}
