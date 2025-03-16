use std::{env, path::Path};

use rusqlite::{Connection, Result as SqliteResult};
use tracing::debug;

pub fn get_db_connection(namespace: &str) -> SqliteResult<Connection> {
    debug!("Getting DB connection for namespace: {}", namespace);

    let sqlite_path = env::var("SQLITE_PATH").unwrap_or("sqlite".into());
    let db_path = format!("{}/{}.db", sqlite_path, namespace);

    if Path::new(&db_path).exists() {
        let conn: Connection = Connection::open(&db_path)?;
        return Ok(conn);
    }

    let conn: Connection = Connection::open(&db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS secrets (
            key TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}
