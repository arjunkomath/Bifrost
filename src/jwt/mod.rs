use std::env;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub namespace: String,
    pub exp: usize,
}

pub fn validate_token(token: &str) -> Result<Claims, String> {
    let key = DecodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref());
    decode::<Claims>(token, &key, &Validation::default())
        .map(|token_data| token_data.claims)
        .map_err(|e| e.to_string())
}

pub fn generate_token(user_id: &str, namespace: &str) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        namespace: namespace.to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    };

    let key = EncodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref());
    encode(&Header::default(), &claims, &key).unwrap()
}
