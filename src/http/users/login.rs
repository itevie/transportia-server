use crate::http::HttpState;
use crate::http::errors::ApiError;
use crate::util;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "Authentication",
    summary = "Log in",
    request_body = LoginBody,
    responses(
        (status = 201, description = "Login successful", body = String),
        (status = 404, description = "Email does not exist or invalid password"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    State(state): State<HttpState>,
    Json(user): Json<LoginBody>,
) -> Result<Response, ApiError> {
    if let None = state
        .db
        .users
        .get_by_email(&user.email, false)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
    {
        return Err(ApiError::EmailDoesNotExist);
    }

    let user = match state
        .db
        .users
        .check_password(&user.email, &user.password)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
    {
        Some(user) => user,
        None => return Err(ApiError::UserNotFound),
    };

    let jwt = util::jwt::create_jwt(user, "my-cute-secret-which-no-one-will-crack")
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok((StatusCode::CREATED, jwt).into_response())
}
