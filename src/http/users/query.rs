use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::db;
use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    summary = "Get a user",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User", body = db::resources::user::User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_user(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let user = state
        .db
        .users
        .get(id, false)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or(ApiError::UserNotFound)?;

    Ok((StatusCode::OK, Json(user)).into_response())
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "Users",
    summary = "Get the current user",
    responses(
        (status = 200, description = "Current user", body = db::resources::user::User),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn me(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser
) -> Result<Response, ApiError> {
    let user = state
        .db
        .users
        .get(user_id, true)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or(ApiError::UserNotFound)?;
    
    Ok((StatusCode::OK, Json(user)).into_response())
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    summary = "Search users",
    params(
        db::users::UserQuery
    ),
    responses(
        (status = 200, description = "Matching users", body = [db::resources::user::User]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_users(
    State(state): State<HttpState>,
    Query(query): Query<db::users::UserQuery>,
) -> Result<Json<Vec<db::resources::user::User>>, String> {
    let listings = state
        .db
        .users
        .search(&query)
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}