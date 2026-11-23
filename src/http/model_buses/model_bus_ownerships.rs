use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resource::model_bus::model_bus_ownership::{
    ModelBusOwnershipQuery, ModelBusOwnerships,
};
use crate::db::resource::model_bus::model_bus_ownership_def::{
    ModelBusOwnership, ModelBusOwnershipFilledOut,
};
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::extractors::ratelimit::{CreateRatelimit, QueryRatelimit, RateLimit, SearchRatelimit};
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ModelBusOwnershipCreateRequest {
    pub model_bus_id: i64,
    pub certificate_number: Option<i64>,
    pub bought_at: Option<String>,
    pub defects: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ModelBusOwnershipUpdateRequest {
    pub certificate_number: Option<i64>,
    pub bought_at: Option<String>,
    pub defects: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModelBusOwnershipCreateResponse {
    pub id: i64,
}

#[utoipa::path(
    get,
    path = "/model-bus-ownerships",
    tag = "Model Bus Ownerships",
    summary = "Search model bus ownerships",
    params(ModelBusOwnershipQuery),
    responses(
        (
            status = 200,
            description = "Model bus ownerships matching the search query",
            body = [ModelBusOwnership]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_model_bus_ownerships(
    State(state): State<HttpState>,
    Query(query): Query<ModelBusOwnershipQuery>,
    _ratelimit: RateLimit<SearchRatelimit>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<ModelBusOwnershipFilledOut>>, ApiError> {
    let mut query = query;
    query.user_id = Some(user_id);

    let ownerships = state
        .db
        .model_bus_ownerships
        .search(&query)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled = ModelBusOwnerships::fill_out(state.db.clone(), ownerships, Some(user_id))
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled))
}

#[utoipa::path(
    get,
    path = "/model-bus-ownerships/{id}",
    tag = "Model Bus Ownerships",
    summary = "Get model bus ownership",
    params(
        ("id" = i64, Path, description = "Ownership ID")
    ),
    responses(
        (
            status = 200,
            description = "Model bus ownership",
            body = ModelBusOwnership
        ),
        (
            status = 404,
            description = "Model bus ownership not found"
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
pub async fn get_model_bus_ownership(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<QueryRatelimit>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<ModelBusOwnershipFilledOut>, ApiError> {
    let ownership = state
        .db
        .model_bus_ownerships
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .filter(|ownership| ownership.user_id == user_id)
        .ok_or_else(|| ApiError::NotFound("Model bus ownership not found".to_string()))?;

    let filled = ModelBusOwnerships::fill_out(state.db.clone(), vec![ownership], Some(user_id))
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled.get(0).unwrap().clone()))
}

#[utoipa::path(
    post,
    path = "/model-bus-ownerships",
    tag = "Model Bus Ownerships",
    summary = "Create a model bus ownership",
    request_body = ModelBusOwnershipCreateRequest,
    responses(
        (
            status = 201,
            description = "Model bus ownership created successfully",
            body = ModelBusOwnershipCreateResponse
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
pub async fn create_model_bus_ownership(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    _ratelimit: RateLimit<CreateRatelimit>,
    Json(body): Json<ModelBusOwnershipCreateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let ownership = ModelBusOwnership {
        id: 0,
        user_id,
        model_bus_id: body.model_bus_id,
        certificate_number: body.certificate_number,
        bought_at: body.bought_at,
        defects: body.defects,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let id = state
        .db
        .model_bus_ownerships
        .create(&ownership)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(ModelBusOwnershipCreateResponse { id }),
    )
        .into_response())
}

#[utoipa::path(
    patch,
    path = "/model-bus-ownerships/{id}",
    tag = "Model Bus Ownerships",
    summary = "Update a model bus ownership",
    params(
        ("id" = i64, Path, description = "Ownership ID")
    ),
    request_body = ModelBusOwnershipUpdateRequest,
    responses(
        (
            status = 204,
            description = "Model bus ownership updated successfully"
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
            description = "Model bus ownership not found"
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
pub async fn update_model_bus_ownership(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<ModelBusOwnershipUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut ownership = state
        .db
        .model_bus_ownerships
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .filter(|ownership| ownership.user_id == user_id)
        .ok_or_else(|| ApiError::NotFound("Model bus ownership not found".to_string()))?;

    if let Some(certificate_number) = body.certificate_number {
        ownership.certificate_number = Some(certificate_number);
    }

    if let Some(bought_at) = body.bought_at {
        ownership.bought_at = Some(bought_at);
    }

    if let Some(defects) = body.defects {
        ownership.defects = Some(defects);
    }

    state
        .db
        .model_bus_ownerships
        .upsert(&ownership)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    delete,
    path = "/model-bus-ownerships/{id}",
    tag = "Model Bus Ownerships",
    summary = "Delete a model bus ownership",
    params(
        ("id" = i64, Path, description = "Ownership ID")
    ),
    responses(
        (
            status = 204,
            description = "Model bus ownership deleted successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not an admin"
        ),
        (
            status = 404,
            description = "Model bus ownership not found"
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
pub async fn delete_model_bus_ownership(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    is_admin(state.db.clone(), user_id).await?;

    state
        .db
        .model_bus_ownerships
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Model bus ownership not found".to_string()))?;

    state
        .db
        .model_bus_ownerships
        .delete(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
