use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::util::jwt::JwtClaims;

pub struct OptionalAuthUser(pub Option<OptionalAuthUserResult>);

pub struct OptionalAuthUserResult {
    pub user_id: i64,
}

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|auth| auth.strip_prefix("Bearer "))
            .and_then(|token| {
                let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

                decode::<JwtClaims>(
                    token,
                    &DecodingKey::from_secret(secret.as_bytes()),
                    &Validation::default(),
                )
                .ok()
            })
            .map(|token_data| OptionalAuthUserResult {
                user_id: token_data.claims.sub,
            });

        Ok(OptionalAuthUser(user))
    }
}
