
use std::{
    marker::PhantomData,
    net::SocketAddr,
};

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts},
    http::request::Parts,
};

use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::{
    http::HttpState,
    util::jwt::JwtClaims,
};

use super::{
    bucket::{RateLimitBucket, RateLimitPolicy},
    state::{RateLimitIdentity, RateLimitState},
};

pub struct RateLimit<B: RateLimitPolicy>(
    PhantomData<B>,
);

impl<B: RateLimitPolicy> RateLimit<B> {
    pub const BUCKET: RateLimitBucket = B::BUCKET;
}

impl<S, B> FromRequestParts<S> for RateLimit<B>
where
    S: Send + Sync,
    HttpState: FromRef<S>,
    B: RateLimitPolicy,
{
    type Rejection = super::rejection::RateLimitRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = HttpState::from_ref(state);

        // Authentication bucket always uses the IP address.
        let identity = if B::BUCKET != RateLimitBucket::Authentication {
            Self::get_user_id(parts, &state.ratelimit)
                .map(RateLimitIdentity::User)
                .or_else(|| Self::get_ip(parts).map(RateLimitIdentity::Ip))
        } else {
            Self::get_ip(parts).map(RateLimitIdentity::Ip)
        };

        let identity = identity.unwrap_or_else(|| {
            RateLimitIdentity::Ip("unknown".to_string())
        });

        match state.ratelimit.check(B::BUCKET, identity) {
            Ok(_) => Ok(Self(PhantomData)),
            Err(result) => Err(
                super::rejection::RateLimitRejection {
                    retry_after: result.retry_after,
                }
            ),
        }
    }
}

impl<B: RateLimitPolicy> RateLimit<B> {
    fn get_user_id(
        parts: &Parts,
        ratelimit: &RateLimitState,
    ) -> Option<i64> {
        let token = parts.headers
            .get("Authorization")?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")?;

        let secret = ratelimit.jwt_secret();

        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(secret),
            &Validation::default(),
        )
            .ok()
            .map(|token_data| token_data.claims.sub)
    }

    fn get_ip(parts: &Parts) -> Option<String> {
        parts.extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect_info| connect_info.0.ip().to_string())
    }
}
