use crate::db::resource::favourite::favourite_def::{Favourite, FavouriteFilled, FavouriteType};
use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::extractors::ratelimit::{FavouriteRatelimit, RateLimit};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct FavouriteFetchQuery {
    pub target_type: FavouriteType,
    pub fill_out: Option<bool>,
}

// TODO: Make it so you can fetch other people's favourites
// TODO: If they have public_favourites enabled
#[utoipa::path(
    get,
    path = "/users/{user_id}/favourites",
    tag = "Favourites",
    summary = "Fetch all favourites given an object type",
    params(FavouriteFetchQuery),
    responses(
        (
            status = 200,
            description = "Favourites retrieved successfully. The object field is included when fill_out is true.",
            content_type = "application/json",
            body = [FavouriteFilled]
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
pub async fn get_favourites(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<FavouriteRatelimit>,
    Query(query): Query<FavouriteFetchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = state
        .db
        .users
        .get(id, false)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;

    if user.id != user_id && user.public_favourites == false {
        return Err(ApiError::Unauthorized(format!("User {} does not have public favourites enabled", id)));
    }

    let favourites = state
        .db
        .favourites
        .get_by_type(id, query.target_type)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    if query.fill_out.unwrap_or(false) {
        let favourites = state
            .db
            .favourites
            .fill(state.db.clone(), favourites, Some(user_id))
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        return Ok(Json(
            serde_json::to_value(favourites)
                .map_err(|err| ApiError::GenericServerError(err.to_string()))?,
        ));
    }

    Ok(Json(serde_json::to_value(favourites).map_err(|err| {
        ApiError::GenericServerError(err.to_string())
    })?))
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct FavouriteGroupFetchQuery {
    pub fill_out: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/users/{user_id}/favourites/group/{group_id}",
    tag = "Favourites",
    summary = "Fetch favourites belonging to a group",
    params(
        ("group_id" = i64, Path, description = "Favourite group ID"),
        FavouriteGroupFetchQuery
    ),
    responses(
        (
            status = 200,
            description = "Favourites retrieved successfully. The object field is included when fill_out is true.",
            content_type = "application/json",
            body = [FavouriteFilled]
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
pub async fn get_favourites_by_group(
    State(state): State<HttpState>,
    AuthUser { user_id: sender_user_id }: AuthUser,
    _ratelimit: RateLimit<FavouriteRatelimit>,
    Path((user_id, group_id)): Path<(i64, i64)>,
    Query(query): Query<FavouriteGroupFetchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = state
        .db
        .users
        .get(user_id, false)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", user_id)))?;

    if user.id != sender_user_id && user.public_favourites == false {
        return Err(ApiError::Unauthorized(format!("User {} does not have public favourites enabled", user_id)));
    }

    let favourites = state
        .db
        .favourites
        .get_by_group_id(user_id, group_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    if query.fill_out.unwrap_or(false) {
        let favourites = state
            .db
            .favourites
            .fill(state.db.clone(), favourites, Some(user_id))
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        return Ok(Json(
            serde_json::to_value(favourites)
                .map_err(|err| ApiError::GenericServerError(err.to_string()))?,
        ));
    }

    Ok(Json(serde_json::to_value(favourites).map_err(|err| {
        ApiError::GenericServerError(err.to_string())
    })?))
}

#[utoipa::path(
    get,
    path = "/users/{user_id}/favourites/{target_type}/{target_id}",
    tag = "Favourites",
    summary = "Get a specific favourite",
    params(
        ("target_type" = FavouriteType, Path, description = "Favourite target type"),
        ("target_id" = i64, Path, description = "Favourite target ID")
    ),
    responses(
        (
            status = 200,
            description = "Favourite retrieved successfully",
            body = Favourite
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
pub async fn get_favourite(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    _ratelimit: RateLimit<FavouriteRatelimit>,
    Path((path_user_id, target_type, target_id)): Path<(i64, FavouriteType, i64)>,
) -> Result<Json<Option<Favourite>>, ApiError> {
    let favourite = state
        .db
        .favourites
        .get(user_id, target_type, target_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(favourite))
}
