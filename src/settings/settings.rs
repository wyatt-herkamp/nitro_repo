use serde::{Deserialize, Serialize};

use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::Mysql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{deserialize, serialize};

use crate::schema::*;

use crate::error::internal_error::InternalError;
use crate::utils::Resources;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralSettings {
    pub name: DBSetting,
    pub installed: DBSetting,
    pub version: DBSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecuritySettings {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailSetting {
    pub email_username: DBSetting,
    pub email_password: DBSetting,
    pub email_host: DBSetting,
    pub encryption: DBSetting,
    pub from: DBSetting,
    pub port: DBSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SettingReport {
    pub email: EmailSetting,
    pub general: GeneralSettings,
    pub security: SecuritySettings,
}

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow, Clone)]
#[sql_type = "Text"]
pub struct Setting {
    pub key: String,
    pub name: String,
    pub default: Option<String>,
    pub optional: Option<bool>,
    pub properties: Option<Vec<String>>,
    pub options: Option<Vec<String>>,
    pub public: Option<bool>,
}

impl FromSql<Text, Mysql> for Setting {
    fn from_sql(
        bytes: Option<&<diesel::mysql::Mysql as Backend>::RawValue>,
    ) -> deserialize::Result<Setting> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes).unwrap();
        let result = SettingManager::get_setting(t);
        Ok(result.unwrap())
    }
}

impl ToSql<Text, Mysql> for Setting {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Mysql>) -> serialize::Result {
        let s = self.key.clone();
        <String as ToSql<Text, Mysql>>::to_sql(&s, out)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "settings"]
pub struct DBSetting {
    pub id: i64,
    pub setting: Setting,
    pub value: String,
    pub updated: i64,
}

impl DBSetting {
    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }
}

pub fn get_file() -> String {
    let cow = Resources::get("settings.toml").unwrap().data;
    
    String::from_utf8(cow.to_vec()).unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub settings: Vec<Setting>,
}

pub struct SettingManager {}

impl SettingManager {
    pub fn get_setting(key: String) -> Option<Setting> {
        let settings = SettingManager::get_settings();
        for setting in settings {
            if setting.key == key {
                return Some(setting);
            }
        }
        None
    }
    pub fn get_settings() -> Vec<Setting> {
        let settings: Settings = toml::from_str(&*get_file()).unwrap();
        settings.settings
    }
}

impl From<String> for Setting {
    fn from(value: String) -> Self {
        SettingManager::get_setting(value).unwrap()
    }
}

impl From<&str> for Setting {
    fn from(value: &str) -> Self {
        SettingManager::get_setting(value.to_string()).unwrap()
    }
}

impl FromStr for Setting {
    type Err = InternalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SettingManager::get_setting(s.to_string())
            .ok_or(InternalError::Error("Missing Setting".to_string()))
    }
}
pub trait SettingVec {
    fn get_setting_by_key(&self, key: &str) -> Option<&DBSetting>;
}
impl SettingVec for Vec<DBSetting> {
    fn get_setting_by_key(&self, key: &str) -> Option<&DBSetting> {
        for x in self {
            if x.setting.key.eq(key) {
                return Option::Some(x);
            }
        }
        None
    }
}
