
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resources::tram_livery::TramLivery;
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TramLiverySearchQuery {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramLiveryCreateRequest {
    pub name: String,
    pub css: Option<String>,
    pub text_colour: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramLiveryUpdateRequest {
    pub name: Option<String>,
    pub css: Option<String>,
    pub text_colour: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TramLiveryCreateResponse {
    pub id: i64,
}

#[utoipa::path(
    get,
    path = "/tram-liveries",
    tag = "Trams",
    summary = "Search tram liveries",
    params(TramLiverySearchQuery),
    responses(
        (
            status = 200,
            description = "Tram liveries matching the search query",
            body = [TramLivery]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_tram_liveries(
    State(state): State<HttpState>,
    Query(query): Query<TramLiverySearchQuery>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<TramLivery>>, ApiError> {
    let liveries = state
        .db
        .tram_liveries
        .search(query.query.as_deref().unwrap_or(""))
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(liveries))
}

#[utoipa::path(
    get,
    path = "/tram-liveries/{id}",
    tag = "Trams",
    summary = "Get tram livery",
    params(
        ("id" = i64, Path, description = "Tram livery ID")
    ),
    responses(
        (
            status = 200,
            description = "Tram livery",
            body = TramLivery
        ),
        (
            status = 404,
            description = "Tram livery not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_tram_livery(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<TramLivery>, ApiError> {
    let livery = state
        .db
        .tram_liveries
        .get(id)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| {
            ApiError::NotFound("Tram livery not found".to_string())
        })?;

    Ok(Json(livery))
}

#[utoipa::path(
    post,
    path = "/tram-liveries",
    tag = "Trams",
    summary = "Create a tram livery",
    request_body = TramLiveryCreateRequest,
    responses(
        (
            status = 201,
            description = "Tram livery created successfully",
            body = TramLiveryCreateResponse
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
pub async fn create_tram_livery(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramLiveryCreateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id)?;

    let livery = TramLivery {
        id: 0,
        name: body.name,
        css: body.css,
        text_colour: body.text_colour,
    };

    let id = state
        .db
        .tram_liveries
        .create(&livery)
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(TramLiveryCreateResponse { id }),
    )
        .into_response())
}

#[utoipa::path(
    patch,
    path = "/tram-liveries/{id}",
    tag = "Trams",
    summary = "Update a tram livery",
    params(
        ("id" = i64, Path, description = "Tram livery ID")
    ),
    request_body = TramLiveryUpdateRequest,
    responses(
        (
            status = 204,
            description = "Tram livery updated successfully"
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
            description = "Tram livery not found"
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
pub async fn update_tram_livery(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramLiveryUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id)?;

    let mut livery = state
        .db
        .tram_liveries
        .get(id)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| {
            ApiError::NotFound("Tram livery not found".to_string())
        })?;

    if let Some(name) = body.name {
        livery.name = name;
    }

    if let Some(css) = body.css {
        livery.css = Some(css);
    }

    if let Some(text_colour) = body.text_colour {
        livery.text_colour = Some(text_colour);
    }

    state
        .db
        .tram_liveries
        .update(&livery)
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    delete,
    path = "/tram-liveries/{id}",
    tag = "Trams",
    summary = "Delete a tram livery",
    params(
        ("id" = i64, Path, description = "Tram livery ID")
    ),
    responses(
        (
            status = 204,
            description = "Tram livery deleted successfully"
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
            description = "Tram livery not found"
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
pub async fn delete_tram_livery(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    is_admin(state.db.clone(), user_id)?;

    state
        .db
        .tram_liveries
        .get(id)
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| {
            ApiError::NotFound("Tram livery not found".to_string())
        })?;

    state
        .db
        .tram_liveries
        .delete(id)
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
