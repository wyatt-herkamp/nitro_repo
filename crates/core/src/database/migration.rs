use sqlx::{migrate::Migrator, PgPool};
static MIGRATOR: Migrator = sqlx::migrate!();
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
