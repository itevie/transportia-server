use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::db::resource::favourite::favourite_group::{FavouriteGroupQuery};
use crate::db::resource::favourite::favourite_group_def::FavouritesGroup;
use crate::extractors::authenticator::AuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[utoipa::path(
    get,
    path = "/favourites-groups",
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
    AuthUser { user_id }: AuthUser,
    Query(query): Query<FavouriteGroupQuery>,
) -> Result<Response, ApiError> {
    let groups = state
        .db
        .favourites_groups
        .search(&query, user_id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(groups)).into_response())
}