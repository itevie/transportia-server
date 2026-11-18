use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resource::tram::tram_type_def::TramType;
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TramTypeSearchQuery {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramTypeCreateRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramTypeUpdateRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TramTypeCreateResponse {
    pub id: i64,
}

#[utoipa::path(
    get,
    path = "/tram-types",
    tag = "Trams",
    summary = "Search tram types",
    params(TramTypeSearchQuery),
    responses(
        (
            status = 200,
            description = "Tram types matching the search query",
            body = [TramType]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_tram_types(
    State(state): State<HttpState>,
    Query(query): Query<TramTypeSearchQuery>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<TramType>>, ApiError> {
    let tram_types = state
        .db
        .tram_types
        .search(query.query.as_deref().unwrap_or(""))
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(tram_types))
}

#[utoipa::path(
    get,
    path = "/tram-types/{id}",
    tag = "Trams",
    summary = "Get tram type",
    params(
        ("id" = i64, Path, description = "Tram type ID")
    ),
    responses(
        (
            status = 200,
            description = "Tram type",
            body = TramType
        ),
        (
            status = 404,
            description = "Tram type not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_tram_type(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<TramType>, ApiError> {
    let tram_type = state
        .db
        .tram_types
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram type not found".to_string()))?;

    Ok(Json(tram_type))
}

#[utoipa::path(
    post,
    path = "/tram-types",
    tag = "Trams",
    summary = "Create a tram type",
    request_body = TramTypeCreateRequest,
    responses(
        (
            status = 201,
            description = "Tram type created successfully",
            body = TramTypeCreateResponse
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
pub async fn create_tram_type(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramTypeCreateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let tram_type = TramType {
        id: 0,
        name: body.name,
    };

    let id = state
        .db
        .tram_types
        .create(&tram_type)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::CREATED, Json(TramTypeCreateResponse { id })).into_response())
}

#[utoipa::path(
    patch,
    path = "/tram-types/{id}",
    tag = "Trams",
    summary = "Update a tram type",
    params(
        ("id" = i64, Path, description = "Tram type ID")
    ),
    request_body = TramTypeUpdateRequest,
    responses(
        (
            status = 204,
            description = "Tram type updated successfully"
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
            description = "Tram type not found"
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
pub async fn update_tram_type(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramTypeUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut tram_type = state
        .db
        .tram_types
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram type not found".to_string()))?;

    if let Some(name) = body.name {
        tram_type.name = name;
    }

    state
        .db
        .tram_types
        .update(&tram_type)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    delete,
    path = "/tram-types/{id}",
    tag = "Trams",
    summary = "Delete a tram type",
    params(
        ("id" = i64, Path, description = "Tram type ID")
    ),
    responses(
        (
            status = 204,
            description = "Tram type deleted successfully"
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
            description = "Tram type not found"
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
pub async fn delete_tram_type(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    is_admin(state.db.clone(), user_id).await?;

    state
        .db
        .tram_types
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram type not found".to_string()))?;

    state
        .db
        .tram_types
        .delete(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
