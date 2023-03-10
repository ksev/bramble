use std::str::FromStr;

use anyhow::Result;
use tokio::sync::OnceCell;

use sqlx::{
    pool::PoolConnection,
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqlitePoolOptions},
    Sqlite, SqlitePool, Transaction,
};

static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

async fn open_pool() -> Result<SqlitePool> {
    let path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "sqlite:database.sqlite3".into());

    let options = SqliteConnectOptions::from_str(&path)?
        .create_if_missing(true)
        .auto_vacuum(SqliteAutoVacuum::Full);

    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .connect_with(options)
        .await?;

    Ok(pool)
}

/**
 * Get a connection from the global db pool
 */
pub async fn connection() -> Result<PoolConnection<Sqlite>> {
    let pool = POOL.get_or_try_init(open_pool).await?;

    let connection = pool.acquire().await?;
    Ok(connection)
}

/**
 * Get a connection from the global db pool
 */
pub async fn begin<'c>() -> Result<Transaction<'c, Sqlite>> {
    let pool = POOL.get_or_try_init(open_pool).await?;

    let connection = pool.begin().await?;
    Ok(connection)
}
