use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::db::resource::user::def::User;

#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: i64,
    pub exp: usize,
}

pub fn create_jwt(user: User, secret: &str) -> jsonwebtoken::errors::Result<String> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 60 * 60 * 24 * 7;

    let claims = JwtClaims {
        sub: user.id,
        exp: exp as usize,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
}