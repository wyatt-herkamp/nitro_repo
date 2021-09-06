use crate::apierror::APIError;
use crate::schema::settings::dsl::settings;
use crate::settings::action::{add_new_setting, get_setting, update_setting};
use crate::settings::settings::DBSetting;
use crate::utils::get_current_time;
use diesel::MysqlConnection;

pub fn quick_add(key: &str, value: String, conn: &MysqlConnection) -> Result<(), APIError> {
    let result = get_setting(key, &conn)?;
    if let Some(mut setting) = result {
        setting.set_value(value.clone());
        update_setting(&setting, &conn)?;
    }
    let setting = DBSetting {
        id: 0,
        setting: key.into(),
        value,
        updated: get_current_time(),
    };
    add_new_setting(&setting, &conn)?;
    return Ok(());
}
