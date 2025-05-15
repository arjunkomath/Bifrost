use std::env;

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn setup_db() -> Result<Pool<Postgres>> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS vault (
            key TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
