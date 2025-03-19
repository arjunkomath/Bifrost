use std::env;

use anyhow::Result;
use libsql::{Builder, Connection};
use tracing::debug;

pub async fn get_db_connection(namespace: &str) -> Result<Connection> {
    debug!("Getting DB connection for namespace: {}", namespace);

    let org = env::var("TURSO_ORG")?;
    let token = env::var("TURSO_GROUP_TOKEN")?;
    let url = format!("https://{}-{}.turso.io", namespace, org);
    let db = Builder::new_remote(url, token).build().await?;

    let conn = db.connect()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS secrets (
            key TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        (),
    )
    .await?;

    Ok(conn)
}
