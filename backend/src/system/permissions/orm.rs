use crate::system::permissions::UserPermissions;
use sea_orm::{DbErr, JsonValue};

impl From<UserPermissions> for JsonValue {
    fn from(auth: UserPermissions) -> Self {
        serde_json::to_value(auth).unwrap()
    }
}

impl From<UserPermissions> for sea_orm::Value {
    fn from(source: UserPermissions) -> Self {
        sea_orm::Value::Json(Some(Box::new(source.into())))
    }
}

impl sea_orm::TryGetable for UserPermissions {
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

impl sea_orm::sea_query::ValueType for UserPermissions {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::Json(Some(x)) => {
                let auth_properties: UserPermissions = serde_json::from_value(*x)
                    .map_err(|_error| sea_orm::sea_query::ValueTypeErr)?;
                Ok(auth_properties)
            }
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(UserPermissions).to_owned()
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::Json
    }
}
