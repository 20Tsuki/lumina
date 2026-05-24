use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs;
use std::path::Path;

pub async fn init(config: &crate::config::Config) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = Path::new(config.db_path()).parent() {
        fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(config.db_path())
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    let migrations_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/db/migrations");
    let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let sql = fs::read_to_string(entry.path())?;
        sqlx::query(&sql).execute(pool).await?;
    }

    tracing::info!("database migrations complete");
    Ok(())
}
