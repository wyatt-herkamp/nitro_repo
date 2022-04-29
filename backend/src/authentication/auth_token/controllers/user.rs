use crate::api_response::{APIResponse, SiteResponse};
use crate::authentication::auth_token::database::TokenProperties;
use crate::authentication::auth_token::{
    generate_token, AuthTokenEntity, AuthTokenModel, TokenResponse,
};
use crate::authentication::{auth_token, Authentication};
use crate::error::internal_error::InternalError::NotFound;
use crate::system::controllers::me::ActiveValue::{NotSet, Set};
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use crate::utils::get_current_time;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, HttpRequest};
use log::error;
use sea_orm::EntityTrait;
use sea_orm::{DatabaseConnection, IntoActiveModel};
use serde::{Deserialize, Serialize};
use time::Duration;

#[delete("/api/auth/token/delete/{token}")]
pub async fn delete_token(
    database: Data<DatabaseConnection>,
    r: HttpRequest,
    auth: Authentication,
    path: Path<i64>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&database).await??;

    let auth_token = path.into_inner();
    let token: AuthTokenModel = AuthTokenEntity::find_by_id(auth_token)
        .one(database.as_ref())
        .await?
        .ok_or(NotFound)?;
    if token.user_id != caller.id {
        //They can delete other tokens if they are not the user
        caller.can_i_edit_users()?;
    }
    let token = token.into_active_model();
    AuthTokenEntity::delete(token)
        .exec(database.as_ref())
        .await?;
    APIResponse::new(true, Some(true)).respond(&r)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewToken {
    /// Time from now. As Millis
    #[serde(default = "default_expiration")]
    pub expiration: i64,
    // Token Properties
    #[serde(default = "default_properties")]
    pub properties: TokenProperties,
}

pub fn default_expiration() -> i64 {
    Duration::days(30).whole_milliseconds() as i64
}

pub fn default_properties() -> TokenProperties {
    TokenProperties {
        description: Some("Authorization Token".to_string()),
    }
}

#[post("/api/auth/user/token/create")]
pub async fn create_token(
    database: Data<DatabaseConnection>,
    r: HttpRequest,
    auth: Authentication,
    token_settings: Json<NewToken>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&database).await??;
    let token = generate_token();
    let response = APIResponse::new(true, Some(token.clone()));
    let database = database.clone();
    actix_web::rt::spawn(async move {
        let database = database;
        let token_settings = token_settings.0;
        let expiration = auth_token::token_expiration(token_settings.expiration);
        let new_token = auth_token::database::ActiveModel {
            id: NotSet,
            token: Set(token),
            expiration: Set(expiration),
            created: Set(get_current_time()),
            user_id: Set(caller.id),
            properties: Set(token_settings.properties),
        };
        if let Err(error) = AuthTokenEntity::insert(new_token)
            .exec(database.as_ref())
            .await
        {
            error!("Unable to add Auth Token Error: {}", error);
        }
    });

    response.respond(&r)
}
#[get("/api/auth/user/token/list")]
pub async fn list_tokens(
    database: Data<DatabaseConnection>,
    r: HttpRequest,
    auth: Authentication,
) -> SiteResponse {
    println!("HEY");
    let caller: UserModel = auth.get_user(&database).await??;

    let user: Vec<TokenResponse> = auth_token::get_tokens_by_user(caller.id, &database).await?;
    APIResponse::new(true, Some(user)).respond(&r)
}
