use crate::error::internal_error::InternalError;
use crate::error::request_error::RequestError;
use crate::settings::action::{add_new_setting, get_setting, get_settings, update_setting};
use crate::settings::controller::default_setting;
use crate::settings::settings::{
    DBSetting, EmailSetting, GeneralSettings, SecuritySettings, SettingReport, SettingVec,
};
use crate::utils::get_current_time;
use diesel::MysqlConnection;

pub fn quick_add(key: &str, value: String, conn: &MysqlConnection) -> Result<(), RequestError> {
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

pub fn get_setting_report(connection: &MysqlConnection) -> Result<SettingReport, InternalError> {
    let vec = get_settings(&connection)?;
    let email = EmailSetting {
        email_username: vec
            .get_setting_by_key("email.username")
            .unwrap_or(&default_setting("email.username")?)
            .clone(),
        email_password: vec
            .get_setting_by_key("email.password")
            .unwrap_or(&default_setting("email.password")?)
            .clone(),
        email_host: vec
            .get_setting_by_key("email.host")
            .unwrap_or(&default_setting("email.host")?)
            .clone(),
        encryption: vec
            .get_setting_by_key("email.encryption")
            .unwrap_or(&default_setting("email.encryption")?)
            .clone(),
        from: vec
            .get_setting_by_key("email.from")
            .unwrap_or(&default_setting("email.from")?)
            .clone(),
        port: vec
            .get_setting_by_key("email.port")
            .unwrap_or(&default_setting("email.port")?)
            .clone(),
    };
    let general = GeneralSettings {
        name: vec
            .get_setting_by_key("name.public")
            .unwrap_or(&default_setting("name.public")?)
            .clone(),
        installed: vec
            .get_setting_by_key("installed")
            .unwrap_or(&default_setting("installed")?)
            .clone(),
        version: vec
            .get_setting_by_key("version")
            .unwrap_or(&default_setting("version")?)
            .clone()
    };
    let security = SecuritySettings {};
    return Ok(SettingReport {
        email,
        general,
        security,
    });
}
