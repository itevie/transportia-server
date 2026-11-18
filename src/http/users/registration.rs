use crate::http::HttpState;
use crate::http::errors::ApiError;
use crate::http::users::validation;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RegistrationBody {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/register",
    tag = "Authentication",
    summary = "Register a new user",
    request_body = RegistrationBody,
    responses(
        (status = 201, description = "User created", body = String),
        (status = 400, description = "Invalid username"),
        (status = 409, description = "Email or username already exists"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn register(
    State(state): State<HttpState>,
    Json(user): Json<RegistrationBody>,
) -> Result<Response, ApiError> {
    if let Some(_) = state
        .db
        .users
        .get_by_email(&user.email, false)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
    {
        return Err(ApiError::EmailExists);
    }

    if let Some(_) = state
        .db
        .users
        .get_by_username(&user.username)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
    {
        return Err(ApiError::UsernameExists);
    }

    if !validation::username(user.username.clone()) {
        return Err(ApiError::InvalidUsername);
    }

    state
        .db
        .users
        .create(&user.email, &user.username, &user.password)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::CREATED, "User created").into_response())
}
