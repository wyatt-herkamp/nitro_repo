use crate::system::action::{add_new_user, get_auth_token, get_user_by_username};
use crate::{action, system};

use crate::siteerror::SiteError;

use crate::system::models::User;
use actix_web::http::HeaderMap;

use chrono::{DateTime, Duration, Local};
use diesel::MysqlConnection;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::Add;

use crate::store::action::{get_category_by_sid, get_item_by_sid};

use crate::schema::settings::columns::setting;
use crate::settings::action::get_setting;
use crate::siteerror::SiteError::{MissingArgument, NotFound};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use lettre::transport::smtp::authentication::Credentials;
use lettre::SmtpTransport;
use rand_core::OsRng;
use rust_embed::RustEmbed;
use std::fs::read;
use std::path::{Path, PathBuf};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources"]
pub struct Resources;

impl Resources {
    pub fn file_get(file: &str) -> Vec<u8> {
        let buf = Path::new("resources").join(file);
        if buf.exists() {
            return read(buf).unwrap();
        } else {
            return Resources::get(file).unwrap().to_vec();
        }
    }
    pub fn file_get_string(file: &str) -> String {
        let vec = Resources::file_get(file);
        return String::from_utf8(vec).unwrap();
    }
}

pub fn installed(conn: &MysqlConnection) -> Result<(), SiteError> {
    let option = get_setting("INSTALLED", &conn)?;
    if option.is_none() {
        return Err(SiteError::UnInstalled);
    }

    return Ok(());
}

pub fn get_current_time() -> i64 {
    Local::now().timestamp_millis()
}

pub fn get_current_date_time() -> String {
    let local: DateTime<Local> = Local::now();
    let format = local.format("%B %d %Y %H:%M");
    format.to_string()
}

pub fn default_expiration() -> i64 {
    let time = Local::now();
    time.add(Duration::days(30)).timestamp_millis()
}

pub fn email_login(conn: &MysqlConnection) -> Result<SmtpTransport, SiteError> {
    let username =
        get_setting("email.username", &conn)?.ok_or(SiteError::from("Invalid Email config"))?;
    let password =
        get_setting("email.password", &conn)?.ok_or(SiteError::from("Invalid Email config"))?;
    let host = get_setting("email.host", &conn)?.ok_or(SiteError::from("Invalid Email config"))?;
    let encryption =
        get_setting("email.encryption", &conn)?.ok_or(SiteError::from("Invalid Email config"))?;
    let creds = Credentials::new(username.value, password.value);

    let mailer = SmtpTransport::relay(host.value.as_str())?
        .credentials(creds)
        .build();
    Ok(mailer)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailChangeRequest {
    pub email_username: Option<String>,
    pub email_password: Option<String>,
    pub email_host: Option<String>,
    pub encryption: Option<String>,
    pub from: Option<String>,
    pub port: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<NewPassword>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewPassword {
    pub password: String,
    pub password_two: String,
}

impl NewPassword {
    pub fn hash(&self) -> Result<String, SiteError> {
        if self.password != self.password_two {
            return Err(SiteError::from("Mismatching Password"));
        }
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password_simple(self.password.as_bytes(), salt.as_ref())
            .unwrap()
            .to_string();
        return Ok(password_hash);
    }
}

pub fn new_user(new_user: NewUser, conn: &MysqlConnection) -> Result<User, SiteError> {
    let username = new_user
        .username
        .ok_or(MissingArgument("Username".into()))?;
    let option = system::action::get_user_by_username(username.clone(), &conn)?;
    if option.is_some() {
        return Err(SiteError::Error("Username Already Exists".into()));
    }
    let email = new_user.email.ok_or(MissingArgument("Email".into()))?;
    let option = system::action::get_user_by_email(email.clone(), &conn)?;
    if option.is_some() {
        return Err(SiteError::from("Email Already Exists"));
    }

    let user = User {
        id: 0,
        username: username.clone(),
        email: email.clone(),
        password: new_user
            .password
            .ok_or(MissingArgument("Missing Password".into()))?
            .hash()?,
        created: get_current_time(),
    };
    add_new_user(&user, &conn)?;
    return Ok(
        get_user_by_username(username, &conn)?.ok_or(SiteError::from("Unable to find new user"))?
    );
}

pub fn generate_category_id(connection: &MysqlConnection) -> Result<String, SiteError> {
    loop {
        let x: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let result = get_category_by_sid(x.clone(), &connection)?;
        if result.is_none() {
            return Ok(x);
        }
    }
}

pub fn generate_item_id(connection: &MysqlConnection) -> Result<String, SiteError> {
    loop {
        let x: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let result = get_item_by_sid(x.clone(), &connection)?;
        if result.is_none() {
            return Ok(x);
        }
    }
}

pub fn generate_auth_token(connection: &MysqlConnection) -> Result<String, SiteError> {
    loop {
        let x: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        let result = get_auth_token(x.clone(), &connection)?;
        if result.is_none() {
            return Ok(x);
        }
    }
}

pub fn get_user_by_header(
    header_map: &HeaderMap,
    conn: &MysqlConnection,
) -> Result<Option<User>, SiteError> {
    let option = header_map.get("Authorization");
    if option.is_none() {
        return Ok(None);
    }
    let x = option.unwrap().to_str();
    if x.is_err() {}
    let header = x.unwrap().to_string();

    let split = header.split(" ").collect::<Vec<&str>>();
    let option = split.get(0);
    if option.is_none() {
        return Ok(None);
    }
    let value = split.get(1);
    if value.is_none() {
        return Ok(None);
    }
    let value = value.unwrap().to_string();
    let key = option.unwrap().to_string();
    if key.eq("Bearer") {
        let result = system::action::get_user_from_auth_token(value, conn)?;
        return Ok(result);
    }
    Ok(None)
}
