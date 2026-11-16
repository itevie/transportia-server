
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resources::tram_operator::TramOperator;
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TramOperatorSearchQuery {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramOperatorCreateRequest {
    pub name: String,
    pub location: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramOperatorUpdateRequest {
    pub name: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TramOperatorCreateResponse {
    pub id: i64,
}

#[utoipa::path(
    get,
    path = "/tram-operators",
    tag = "Trams",
    summary = "Search tram operators",
    params(TramOperatorSearchQuery),
    responses(
        (
            status = 200,
            description = "Tram operators matching the search query",
            body = [TramOperator]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_tram_operators(
    State(state): State<HttpState>,
    Query(query): Query<TramOperatorSearchQuery>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<TramOperator>>, ApiError> {
    let operators = state
        .db
        .tram_operators
        .search(query.query.as_deref().unwrap_or(""))
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(operators))
}

#[utoipa::path(
    get,
    path = "/tram-operators/{id}",
    tag = "Trams",
    summary = "Get tram operator",
    params(
        ("id" = i64, Path, description = "Tram operator ID")
    ),
    responses(
        (
            status = 200,
            description = "Tram operator",
            body = TramOperator
        ),
        (
            status = 404,
            description = "Tram operator not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_tram_operator(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<TramOperator>, ApiError> {
    let operator = state
        .db
        .tram_operators
        .get(id)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| {
            ApiError::NotFound("Tram operator not found".to_string())
        })?;

    Ok(Json(operator))
}

#[utoipa::path(
    post,
    path = "/tram-operators",
    tag = "Trams",
    summary = "Create a tram operator",
    request_body = TramOperatorCreateRequest,
    responses(
        (
            status = 201,
            description = "Tram operator created successfully",
            body = TramOperatorCreateResponse
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_tram_operator(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramOperatorCreateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id)?;

    let operator = TramOperator {
        id: 0,
        name: body.name,
        location: body.location,
        website: body.website,
    };

    let id = state
        .db
        .tram_operators
        .create(&operator)
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(TramOperatorCreateResponse { id }),
    )
        .into_response())
}

#[utoipa::path(
    patch,
    path = "/tram-operators/{id}",
    tag = "Trams",
    summary = "Update a tram operator",
    params(
        ("id" = i64, Path, description = "Tram operator ID")
    ),
    request_body = TramOperatorUpdateRequest,
    responses(
        (
            status = 204,
            description = "Tram operator updated successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 404,
            description = "Tram operator not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_tram_operator(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramOperatorUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id)?;

    let mut operator = state
        .db
        .tram_operators
        .get(id)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| {
            ApiError::NotFound("Tram operator not found".to_string())
        })?;

    if let Some(name) = body.name {
        operator.name = name;
    }

    if let Some(location) = body.location {
        operator.location = Some(location);
    }

    if let Some(website) = body.website {
        operator.website = Some(website);
    }

    state
        .db
        .tram_operators
        .update(&operator)
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    delete,
    path = "/tram-operators/{id}",
    tag = "Trams",
    summary = "Delete a tram operator",
    params(
        ("id" = i64, Path, description = "Tram operator ID")
    ),
    responses(
        (
            status = 204,
            description = "Tram operator deleted successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 404,
            description = "Tram operator not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_tram_operator(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    is_admin(state.db.clone(), user_id)?;

    state
        .db
        .tram_operators
        .get(id)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| {
            ApiError::NotFound("Tram operator not found".to_string())
        })?;

    state
        .db
        .tram_operators
        .delete(id)
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
