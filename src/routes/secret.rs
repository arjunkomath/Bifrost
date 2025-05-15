use actix_web::{delete, get, post, web, Responder, Result};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::info;

use crate::{crypto, AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSecretPayload {
    pub data: String,
}

#[derive(Serialize)]
pub struct SecretResponse {
    pub key: String,
}

#[post("/{key}")]
pub async fn upsert(
    payload: web::Json<CreateSecretPayload>,
    key: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let key = key.into_inner();

    info!("Upserting secret for key: {}", key);

    let (encrypted_data, tag) = crypto::encrypt(payload.data.clone()).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to save secret")
    })?;

    let combined_data = format!("{encrypted_data}:{tag}");
    let created_at = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO vault (key, data, created_at) VALUES ($1, $2, $3) ON CONFLICT (key) DO UPDATE SET data = EXCLUDED.data, created_at = EXCLUDED.created_at")
        .bind(&key)
        .bind(&combined_data)
        .bind(&created_at)
        .execute(&state.db)
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError(
                "Internal Error: Failed to execute statement",
            )
        })?;

    Ok(web::Json(SecretResponse { key }))
}

#[derive(Serialize)]
pub struct TokenDataResponse {
    pub data: String,
}

#[get("/{key}")]
pub async fn get(key: web::Path<String>, state: web::Data<AppState>) -> Result<impl Responder> {
    let key = key.into_inner();

    info!("Getting secret for key: {}", key);

    let row = sqlx::query("SELECT data FROM vault WHERE key = $1")
        .bind(&key)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError("Internal Error: DB query failed")
        })?;

    let row = row.ok_or_else(|| actix_web::error::ErrorNotFound("Secret not found"))?;
    let encrypted_data: String = row.try_get("data").map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to get secret")
    })?;

    let result = crypto::decrypt(encrypted_data).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to get secret")
    })?;

    Ok(web::Json(TokenDataResponse { data: result }))
}

#[delete("/{key}")]
pub async fn delete(key: web::Path<String>, state: web::Data<AppState>) -> Result<impl Responder> {
    let key = key.into_inner();

    info!("Deleting secret for key: {}", key);

    let result = sqlx::query("DELETE FROM vault WHERE key = $1")
        .bind(&key)
        .execute(&state.db)
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError(
                "Internal Error: Failed to execute statement",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("Secret not found"));
    }

    Ok(web::Json(SecretResponse { key }))
}
