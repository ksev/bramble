use std::str::FromStr;

use anyhow::Result;
use tokio::sync::OnceCell;

use sqlx::{
    sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool, pool::PoolConnection, Sqlite, Transaction,
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
 * Get a connection from the global db pool
 */
pub async fn connection() -> Result<PoolConnection<Sqlite>> {
    let pool = POOL.get_or_try_init(open_pool)
        .await?;

    let connection = pool.acquire().await?;
    Ok(connection)
}

/**
 * Get a connection from the global db pool
 */
pub async fn begin<'c>() -> Result<Transaction<'c, Sqlite>> {
    let pool = POOL.get_or_try_init(open_pool)
        .await?;

    let connection = pool.begin().await?;
    Ok(connection)
}
