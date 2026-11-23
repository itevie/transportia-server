
use std::time::Duration;

use axum::{
    Json,
    http::{
        HeaderValue,
        StatusCode,
        header::RETRY_AFTER,
    },
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub struct RateLimitRejection {
    pub retry_after: Duration,
}

#[derive(Serialize)]
struct RateLimitErrorResponse {
    error: &'static str,
    message: &'static str,
    retry_after: u64,
}

impl IntoResponse for RateLimitRejection {
    fn into_response(self) -> Response {
        let retry_after = self.retry_after
            .as_secs()
            .max(1);

        let body = RateLimitErrorResponse {
            error: "RATE_LIMIT_EXCEEDED",
            message: "You are making requests too quickly.",
            retry_after,
        };

        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(body),
        ).into_response();

        response.headers_mut().insert(
            RETRY_AFTER,
            HeaderValue::from(retry_after),
        );

        response
    }
}
