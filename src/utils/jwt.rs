// src/utils/jwt.rs
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};

// ✅ ДОБАВИТЬ #[derive(Clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // user id
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub fn create_token(user_id: String, username: String, role: String, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now() + Duration::hours(24);

    let claims = Claims {
        sub: user_id,
        username,
        role,
        exp: expiration.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
