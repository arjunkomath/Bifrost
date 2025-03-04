use actix_web::{get, post, web, HttpMessage, HttpRequest, Responder, Result};
use nanoid::nanoid;
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};

use crate::crypto;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSecretPayload {
    pub data: String,
}

#[derive(Serialize)]
pub struct SecretResponse {
    pub key: String,
}

fn get_db_connection(namespace: &str) -> SqliteResult<Connection> {
    let conn: Connection = Connection::open(format!("sqlite/{}.db", namespace))?;

    // Create table if it doesn't exist
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

fn get_id() -> String {
    let alphabet: [char; 62] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];

    nanoid!(28, &alphabet)
}

#[post("")]
pub async fn create(
    payload: web::Json<CreateSecretPayload>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let (encrypted_data, tag) = crypto::encrypt(payload.data.clone()).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to tokenize data")
    })?;

    let extensions = req.extensions();
    let token = extensions
        .get::<crate::jwt::Claims>()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No token found"))?;

    let conn = get_db_connection(&token.namespace).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to database")
    })?;

    let key = ["sk", &get_id()].join("_");
    let combined_data = [encrypted_data, tag].join("");
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO secrets (key, data, created_at, user_id) VALUES (?1, ?2, ?3, ?4)",
        [&key, &combined_data, &created_at, &token.sub],
    )
    .map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to save secret")
    })?;

    Ok(web::Json(SecretResponse { key: key.clone() }))
}

#[derive(Serialize)]
pub struct TokenDataResponse {
    pub data: String,
}

#[get("/{id}")]
pub async fn get(req: HttpRequest, path: web::Path<String>) -> Result<impl Responder> {
    let path = path.into_inner();

    let extensions = req.extensions();
    let token = extensions
        .get::<crate::jwt::Claims>()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No token found"))?;

    let conn = get_db_connection(&token.namespace).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to database")
    })?;

    let encrypted_data: String = conn
        .query_row(
            "SELECT data FROM secrets WHERE key = ?1 AND user_id = ?2",
            [&path, &token.sub],
            |row| row.get(0),
        )
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid secret key"))?;

    let result = crypto::decrypt(encrypted_data).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to get secret")
    })?;

    Ok(web::Json(TokenDataResponse { data: result }))
}
