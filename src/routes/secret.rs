use actix_web::{get, post, web, Responder, Result};
use serde::{Deserialize, Serialize};

use crate::{crypto, utils::db::get_db_connection};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSecretPayload {
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceQuery {
    pub namespace: String,
}

#[derive(Serialize)]
pub struct SecretResponse {
    pub key: String,
}

#[post("/{key}")]
pub async fn create(
    payload: web::Json<CreateSecretPayload>,
    key: web::Path<String>,
    query: web::Query<NamespaceQuery>,
) -> Result<impl Responder> {
    let key = key.into_inner();

    let (encrypted_data, tag) = crypto::encrypt(payload.data.clone()).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to tokenize data")
    })?;

    let conn = get_db_connection(&query.namespace).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to database")
    })?;

    let combined_data = [encrypted_data, tag].join("");
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO secrets (key, data, created_at) VALUES (?1, ?2, ?3)",
        [&key, &combined_data, &created_at],
    )
    .map_err(|err| {
        actix_web::error::ErrorInternalServerError(format!(
            "Internal Error: Failed to save secret: {}",
            err
        ))
    })?;

    Ok(web::Json(SecretResponse { key: key.clone() }))
}

#[derive(Serialize)]
pub struct TokenDataResponse {
    pub data: String,
}

#[get("/{key}")]
pub async fn get(
    key: web::Path<String>,
    query: web::Query<NamespaceQuery>,
) -> Result<impl Responder> {
    let key = key.into_inner();

    let conn = get_db_connection(&query.namespace).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to database")
    })?;

    let encrypted_data: String = conn
        .query_row("SELECT data FROM secrets WHERE key = ?1", [&key], |row| {
            row.get(0)
        })
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid secret key"))?;

    let result = crypto::decrypt(encrypted_data).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to get secret")
    })?;

    Ok(web::Json(TokenDataResponse { data: result }))
}
