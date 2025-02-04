use sqlx::{PgPool, migrate::Migrator};
static MIGRATOR: Migrator = sqlx::migrate!();
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
