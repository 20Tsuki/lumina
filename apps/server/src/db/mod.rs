pub async fn init(config: &crate::config::Config) -> anyhow::Result<sqlx::SqlitePool> { let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?; Ok(pool) }
