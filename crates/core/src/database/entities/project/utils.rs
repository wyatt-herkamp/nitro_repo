use pg_extended_sqlx_queries::prelude::*;

use sqlx::PgPool;
use uuid::Uuid;

use super::{DBProject, DBProjectColumn};

pub async fn does_project_id_exist(id: Uuid, database: &PgPool) -> Result<bool, sqlx::Error> {
    let result = SelectExists::new(DBProject::table_name())
        .filter(DBProjectColumn::Id.equals(id))
        .execute(database)
        .await?;

    Ok(result)
}
