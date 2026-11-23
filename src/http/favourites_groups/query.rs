use crate::db::resource::favourite::favourite_group::FavouriteGroupQuery;
use crate::db::resource::favourite::favourite_group_def::FavouritesGroup;
use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::extractors::ratelimit::{FavouriteRatelimit, RateLimit};

#[utoipa::path(
    get,
    path = "/users/{user_id}/favourites/groups",
    tag = "Favourite Groups",
    summary = "Fetch favourite groups",
    description = "Fetches favourite groups with optional filtering and pagination.",
    params(
        FavouriteGroupQuery
    ),
    responses(
        (
            status = 200,
            description = "Favourite groups fetched successfully",
            body = Vec<FavouritesGroup>
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn fetch_favourite_groups(
    State(state): State<HttpState>,
    _ratelimit: RateLimit<FavouriteRatelimit>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
    Query(query): Query<FavouriteGroupQuery>,
) -> Result<Response, ApiError> {
    let user = state
        .db
        .users
        .get(id, false)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;

    if user.id != user_id && user.public_favourites == false {
        return Err(ApiError::Unauthorized(format!("User {} does not have public favourites enabled", id)));
    }

    let groups = state
        .db
        .favourites_groups
        .search(&query, user_id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(groups)).into_response())
}
