use anyhow::Result;
use once_cell::sync::Lazy;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use rusqlite::Params;

static DB: Lazy<Pool<SqliteConnectionManager>> = Lazy::new(|| {
    let manager = SqliteConnectionManager::file("rome.db");
    Pool::new(manager).unwrap() 
});

pub async fn execute<P>(sql: &str, params: P) -> Result<usize> where P: Params {
    let out = tokio::task::block_in_place(|| {
        let conn = DB.get()?;
        anyhow::Ok(conn.execute(sql, params))
    })?;

    Ok(out?)
}
