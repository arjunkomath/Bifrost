use actix_web::{delete, get, post, web, Responder, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

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
pub async fn upsert(
    payload: web::Json<CreateSecretPayload>,
    key: web::Path<String>,
    query: web::Query<NamespaceQuery>,
) -> Result<impl Responder> {
    let key = key.into_inner();

    info!("Upserting secret for key: {}", key);

    let (encrypted_data, tag) = crypto::encrypt(payload.data.clone()).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to tokenize data")
    })?;

    let conn = get_db_connection(&query.namespace).await.map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to database")
    })?;

    let combined_data = [encrypted_data, tag].join("");
    let created_at = chrono::Utc::now().to_rfc3339();

    let mut stmt = conn
        .prepare("INSERT OR REPLACE INTO secrets (key, data, created_at) VALUES (?1, ?2, ?3)")
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError(
                "Internal Error: Failed to prepare statement",
            )
        })?;

    stmt.execute([key.as_str(), combined_data.as_str(), created_at.as_str()])
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError(
                "Internal Error: Failed to execute statement",
            )
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

    info!("Getting secret for key: {}", key);

    let conn = get_db_connection(&query.namespace).await.map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to database")
    })?;

    let mut stmt = conn
        .prepare("SELECT data FROM secrets WHERE key = ?1")
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError(
                "Internal Error: Failed to prepare statement",
            )
        })?;

    let mut results = stmt
        .query([key.as_str()])
        .await
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid secret key"))?;

    let result = results
        .next()
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError("Internal Error: Failed to get secret")
        })?
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid secret key"))?;

    let encrypted_data = result.get_str(0).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to get secret")
    })?;

    let result = crypto::decrypt(encrypted_data.to_string()).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to get secret")
    })?;

    Ok(web::Json(TokenDataResponse { data: result }))
}

#[delete("/{key}")]
pub async fn delete(
    key: web::Path<String>,
    query: web::Query<NamespaceQuery>,
) -> Result<impl Responder> {
    let key = key.into_inner();

    info!("Deleting secret for key: {}", key);

    let conn = get_db_connection(&query.namespace).await.map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to database")
    })?;

    let mut stmt = conn
        .prepare("DELETE FROM secrets WHERE key = ?1")
        .await
        .map_err(|_| {
            actix_web::error::ErrorInternalServerError(
                "Internal Error: Failed to prepare statement",
            )
        })?;

    stmt.execute([key.as_str()]).await.map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to execute statement")
    })?;

    Ok(web::Json(SecretResponse { key: key.clone() }))
}
