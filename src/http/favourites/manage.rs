use axum::extract::{Path, Query, State};
use axum::http::{Response, StatusCode};
use axum::Json;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::resource::favourite::favourite_def::FavouriteType;
use crate::extractors::authenticator::AuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct FavouriteQuery {
    pub target_type: FavouriteType,
    pub target_id: i64,
}

#[utoipa::path(
    post,
    path = "/favourites",
    tag = "Favourites",
    summary = "Create a favourite for an object",
    params(FavouriteQuery),
    responses(
        (
            status = 204,
            description = "Favourite created successfully"
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
pub async fn create_favourite(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Query(query): Query<FavouriteQuery>,
) -> Result<StatusCode, ApiError> {
    state
        .db
        .favourites
        .create(
            user_id,
            query.target_id,
            query.target_type,
        )
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/favourites",
    tag = "Favourites",
    summary = "Remove a favourite for an object",
    params(FavouriteQuery),
    responses(
        (
            status = 204,
            description = "Favourite deleted successfully"
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
pub async fn delete_favourite(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Query(query): Query<FavouriteQuery>,
) -> Result<StatusCode, ApiError> {
    state
        .db
        .favourites
        .delete(
            user_id,
            query.target_id,
            query.target_type,
        )
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFavouriteGroupRequest {
    pub group_id: Option<i64>,
}

#[utoipa::path(
    patch,
    path = "/favourites/group/{target_type}/{target_id}",
    tag = "Favourites",
    summary = "Update the group of a favourite",
    params(
        ("target_type" = FavouriteType, Path, description = "Favourite target type"),
        ("target_id" = i64, Path, description = "Favourite target ID")
    ),
    request_body = UpdateFavouriteGroupRequest,
    responses(
        (
            status = 200,
            description = "Favourite group updated successfully",
            body = bool
        ),
        (
            status = 404,
            description = "Favourite not found"
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
pub async fn update_favourite_group(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path((target_type, target_id)): Path<(FavouriteType, i64)>,
    Json(request): Json<UpdateFavouriteGroupRequest>,
) -> Result<Json<bool>, ApiError> {
    let updated = state
        .db
        .favourites
        .update_group_id(
            user_id,
            target_id,
            target_type,
            request.group_id,
        )
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    if !updated {
        return Err(ApiError::NotFound("Could not find favourite".to_string()));
    }

    Ok(Json(true))
}
