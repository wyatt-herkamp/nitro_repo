use std::convert::Infallible;

use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};

use sea_orm::entity::prelude::*;
use sea_orm::JsonValue;
use serde::{Deserialize, Serialize};

use crate::system::user::{UserEntity, UserModel};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenProperties {
    pub description: Option<String>,
}

impl From<TokenProperties> for JsonValue {
    fn from(auth: TokenProperties) -> Self {
        serde_json::to_value(auth).unwrap()
    }
}

impl From<TokenProperties> for sea_orm::Value {
    fn from(source: TokenProperties) -> Self {
        sea_orm::Value::Json(Some(Box::new(source.into())))
    }
}

impl sea_orm::TryGetable for TokenProperties {
    fn try_get(
        res: &sea_orm::QueryResult,
        pre: &str,
        col: &str,
    ) -> Result<Self, sea_orm::TryGetError> {
        let val: JsonValue = res.try_get(pre, col).map_err(sea_orm::TryGetError::DbErr)?;
        serde_json::from_value(val)
            .map_err(|e| sea_orm::TryGetError::DbErr(DbErr::Json(e.to_string())))
    }
}

impl sea_orm::sea_query::ValueType for TokenProperties {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::Json(Some(x)) => {
                let auth_properties: TokenProperties = serde_json::from_value(*x)
                    .map_err(|_error| sea_orm::sea_query::ValueTypeErr)?;
                Ok(auth_properties)
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
    pub properties: TokenProperties,
    pub created: i64,
    pub user_id: i64,
}

impl Model {
    pub async fn get_user(
        &self,
        database: &DatabaseConnection,
    ) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find_by_id(self.user_id).one(database).await
    }
}

impl FromRequest for Model {
    type Error = Infallible;
    type Future = Ready<Result<Model, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<Model>().cloned().unwrap();
        ready(Ok(model))
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::system::user::database::Entity",
        from = "Column::UserId",
        to = "crate::system::user::database::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<crate::system::user::database::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
