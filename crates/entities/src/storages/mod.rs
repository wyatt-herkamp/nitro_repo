use nr_core::user::permissions::UserPermissions;
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;
use sea_orm_exports::SeaORMExports;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize, SeaORMExports)]
#[sea_orm(table_name = "storages")]
#[exports(DBStorage, has_relation)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub config: Json,
    pub created: DateTimeWithTimeZone,
}
impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}