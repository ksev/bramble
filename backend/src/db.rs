use std::str::FromStr;

use anyhow::Result;
use tokio::sync::OnceCell;

use sqlx::{
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

async fn open_pool() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite:rome.sqlite3")?
        .create_if_missing(true)
        .auto_vacuum(SqliteAutoVacuum::Full);
        
    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .connect_with(options)
        .await?;

    Ok(pool)
}

/**
 * Get the global database connection pool
 */
pub async fn pool() -> &'static SqlitePool {
    POOL.get_or_try_init(open_pool)
        .await
        .expect("Could not open database")
}
