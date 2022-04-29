use std::ops::Add;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use actix_web::dev::Payload;
use futures_util::future::{ready, Ready};
use rand::distributions::Alphanumeric;
use rand::Rng;
use sea_orm::entity::prelude::*;
use sea_orm::JsonValue;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use crate::error::internal_error::InternalError;
use crate::Mode;
use crate::system::{auth_token, AuthToken, User, user};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Authentication,
    SessionToken,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthProperties {
    pub description: Option<String>,
    pub token_type: TokenType,

}


impl From<AuthProperties> for JsonValue {
    fn from(auth: AuthProperties) -> Self {
        serde_json::to_value(auth).unwrap()
    }
}

impl From<AuthProperties> for sea_orm::Value {
    fn from(source: AuthProperties) -> Self {
        sea_orm::Value::Json(Some(Box::new(source.into())))
    }
}

impl sea_orm::TryGetable for AuthProperties {
    fn try_get(
        res: &sea_orm::QueryResult,
        pre: &str,
        col: &str,
    ) -> Result<Self, sea_orm::TryGetError> {
        let val: JsonValue = res.try_get(pre, col).map_err(sea_orm::TryGetError::DbErr)?;
        return serde_json::from_value(val).map_err(|e| { sea_orm::TryGetError::DbErr(DbErr::Json(e.to_string())) });
    }
}


impl sea_orm::sea_query::ValueType for AuthProperties {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::Json(Some(x)) => {
                let auth_properties: AuthProperties = serde_json::from_value(*x).map_err(|error| {
                    sea_orm::sea_query::ValueTypeErr
                })?;
                return Ok(auth_properties);
            }
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(AuthProperties).to_owned()
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::Json
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "auth_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub token: String,
    pub expiration: i64,
    pub properties: AuthProperties,
    pub created: i64,
    pub user_id: i64,
}

impl Model {
    pub async fn get_user(&self, database: &DatabaseConnection) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find_by_id(self.user_id).one(database).await
    }
}

impl FromRequest for Model {
    type Error = InternalError;
    type Future = Ready<Result<Model, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<Model>().cloned();
        if model.is_none() {
            return ready(Err(InternalError::Error("Dont Fail".to_string())));
        }

        ready(Ok(model.unwrap()))
    }
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::user::Entity",
    from = "Column::UserId",
    to = "super::user::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_by_token(token: &str, connection: &DatabaseConnection) -> Result<Option<Model>, InternalError> {
    auth_token::Entity::find().filter(auth_token::Column::Token.eq(token)).one(connection).await.map_err(|e| InternalError::DBError(e))
}

pub async fn delete_by_token(token: &str, connection: &DatabaseConnection) -> Result<(), InternalError> {
    auth_token::Entity::delete_many().filter(auth_token::Column::Token.eq(token)).exec(connection).await?;
    Ok(())
}

pub fn generate_token() -> String {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();
    format!("nrp_{}", token)
}

pub fn token_expiration() -> i64 {
    OffsetDateTime::now_utc().add(Duration::days(1)).unix_timestamp()
}