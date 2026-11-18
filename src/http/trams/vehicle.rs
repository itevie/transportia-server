use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resource::tram::tram::{TramQuery, Trams};
use crate::db::resource::tram::tram_def::{Tram, TramFilledOut};
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TramSearchQuery {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramCreateRequest {
    pub operator_id: Option<i64>,
    pub fleet: String,
    pub livery_id: Option<i64>,
    pub tram_type_id: Option<i64>,
    pub name: Option<String>,
    pub branding: Option<String>,
    pub garage: Option<String>,
    pub withdrawn: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramUpdateRequest {
    pub operator: Option<i64>,
    pub fleet: Option<String>,
    pub livery: Option<i64>,
    pub tram_type: Option<i64>,
    pub name: Option<String>,
    pub branding: Option<String>,
    pub garage: Option<String>,
    pub withdrawn: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramCreateResponse {
    pub id: i64,
}

#[utoipa::path(
    get,
    path = "/trams",
    tag = "Trams",
    summary = "Search trams",
    params(TramQuery),
    responses(
        (
            status = 200,
            description = "Trams matching the search query",
            body = [TramFilledOut]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_trams(
    State(state): State<HttpState>,
    Query(query): Query<TramQuery>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Result<Json<Vec<TramFilledOut>>, ApiError> {
    let trams = state
        .db
        .trams
        .search(&query)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = Trams::fill_out(state.db.clone(), trams, user.map(|x| x.user_id))
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out))
}

#[utoipa::path(
    get,
    path = "/trams/{id}",
    tag = "Trams",
    summary = "Get tram",
    params(
        ("id" = i64, Path, description = "Tram ID")
    ),
    responses(
        (
            status = 200,
            description = "Tram",
            body = TramFilledOut
        ),
        (
            status = 404,
            description = "Tram not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_tram(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Result<Json<TramFilledOut>, ApiError> {
    let tram = state
        .db
        .trams
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram not found".to_string()))?;

    let filled_out = Trams::fill_out(state.db.clone(), vec![tram], user.map(|x| x.user_id))
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out.get(0).unwrap().clone()))
}

#[utoipa::path(
    post,
    path = "/trams",
    tag = "Trams",
    summary = "Create a tram",
    request_body = TramCreateRequest,
    responses(
        (
            status = 201,
            description = "Tram created successfully",
            body = TramCreateResponse
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
pub async fn create_tram(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramCreateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let tram = Tram {
        id: 0,
        operator_id: body.operator_id,
        fleet: body.fleet,
        livery_id: body.livery_id,
        tram_type_id: body.tram_type_id,
        name: body.name,
        garage: body.garage,
        withdrawn: body.withdrawn,
        branding: body.branding,
    };

    let id = state
        .db
        .trams
        .create(&tram)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::CREATED, Json(TramCreateResponse { id })).into_response())
}

#[utoipa::path(
    patch,
    path = "/trams/{id}",
    tag = "Trams",
    summary = "Update a tram",
    params(
        ("id" = i64, Path, description = "Tram ID")
    ),
    request_body = TramUpdateRequest,
    responses(
        (
            status = 204,
            description = "Tram updated successfully"
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
            description = "Tram not found"
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
pub async fn update_tram(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut tram = state
        .db
        .trams
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram not found".to_string()))?;

    if let Some(operator) = body.operator {
        tram.operator_id = Some(operator);
    }

    if let Some(fleet) = body.fleet {
        tram.fleet = fleet;
    }

    if let Some(livery) = body.livery {
        tram.livery_id = Some(livery);
    }

    if let Some(tram_type) = body.tram_type {
        tram.tram_type_id = Some(tram_type);
    }

    if let Some(name) = body.name {
        tram.name = Some(name);
    }

    if let Some(branding) = body.branding {
        tram.branding = Some(branding);
    }

    if let Some(withdrawn) = body.withdrawn {
        tram.withdrawn = withdrawn;
    }

    if let Some(garage) = body.garage {
        tram.garage = Some(garage);
    }

    state
        .db
        .trams
        .update(&tram)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    delete,
    path = "/trams/{id}",
    tag = "Trams",
    summary = "Delete a tram",
    params(
        ("id" = i64, Path, description = "Tram ID")
    ),
    responses(
        (
            status = 204,
            description = "Tram deleted successfully"
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
            description = "Tram not found"
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
pub async fn delete_tram(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    is_admin(state.db.clone(), user_id).await?;

    state
        .db
        .trams
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram not found".to_string()))?;

    state
        .db
        .trams
        .delete(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
