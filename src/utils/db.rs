use std::{env, path::Path};

use rusqlite::{Connection, Result as SqliteResult};

pub fn get_db_connection(namespace: &str) -> SqliteResult<Connection> {
    let sqlite_path = env::var("SQLITE_PATH").unwrap_or("sqlite".into());
    let db_path = format!("{}/{}.db", sqlite_path, namespace);

    let conn: Connection = Connection::open(&db_path)?;

    if Path::new(&db_path).exists() {
        return Ok(conn);
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS secrets (
            user_id TEXT NOT NULL,
            key TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}
