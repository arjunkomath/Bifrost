use actix_web::{post, web, Responder, Result};
use serde::{Deserialize, Serialize};

use crate::utils::{db::get_db_connection, jwt::generate_token};
#[derive(Debug, Clone, Deserialize)]
struct CreateTokenPayload {
    pub user_id: String,
    pub namespace: String,
}

#[derive(Serialize, Deserialize)]
struct TokenResponse {
    pub jwt: String,
}

#[post("/token")]
pub async fn create_token(payload: web::Json<CreateTokenPayload>) -> Result<impl Responder> {
    if payload.user_id.is_empty() || payload.namespace.is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Namespace and User ID is required",
        ));
    }

    if !payload
        .namespace
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-')
    {
        return Err(actix_web::error::ErrorBadRequest(
            "Namespace must be alphanumeric or contain hyphens",
        ));
    }

    let jwt = generate_token(&payload.user_id, &payload.namespace);

    get_db_connection(&payload.namespace).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to create database")
    })?;

    Ok(web::Json(TokenResponse { jwt }))
}
