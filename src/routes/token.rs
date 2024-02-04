use actix_web::{get, put, web, Responder, Result};
use nanoid::nanoid;
use redis::Commands;
use serde::{Deserialize, Serialize};

use crate::crypto;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTokenPayload {
    pub data: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub key: String,
}

fn get_redis_connection() -> anyhow::Result<redis::Connection> {
    let client = redis::Client::open("redis://127.0.0.1:6379")?;
    let con = client.get_connection()?;

    Ok(con)
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

#[put("/token")]
pub async fn create(payload: web::Json<CreateTokenPayload>) -> Result<impl Responder> {
    let (encrypted_data, tag) = crypto::encrypt(payload.data.clone()).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to tokenize data")
    })?;

    let mut con = get_redis_connection().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to redis")
    })?;

    let key = ["tok", &get_id()].join("_");
    con.set(&key, [encrypted_data, tag].join("")).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to save token")
    })?;

    Ok(web::Json(TokenResponse { key: key.clone() }))
}

#[derive(Serialize)]
pub struct TokenDataResponse {
    pub data: String,
}

#[get("/token/{key}")]
pub async fn get(path: web::Path<String>) -> Result<impl Responder> {
    let path = path.into_inner();

    let mut con = get_redis_connection().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to connect to redis")
    })?;

    let encrypted_data: String = con
        .get(&path)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid token key"))?;

    let result = crypto::decrypt(encrypted_data).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Internal Error: Failed to detokenize data")
    })?;

    Ok(web::Json(TokenDataResponse { data: result }))
}
