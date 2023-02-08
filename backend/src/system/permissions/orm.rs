use crate::system::permissions::UserPermissions;
use sea_orm::sea_query::ArrayType;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveValue, ColIdx, DbErr, IntoActiveValue, JsonValue, QueryResult, TryGetError};

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
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let val: JsonValue = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        serde_json::from_value(val).map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string())))
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

    fn array_type() -> ArrayType {
        ArrayType::Json
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::Json
    }
}
impl IntoActiveValue<UserPermissions> for UserPermissions {
    fn into_active_value(self) -> ActiveValue<UserPermissions> {
        Set(self)
    }
}
