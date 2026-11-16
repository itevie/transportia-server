use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::auth::quick::check_favourites_group;
use crate::db::favourites_groups::FavouriteGroupQuery;
use crate::db::resources::favourites_group::FavouritesGroup;
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
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(groups)).into_response())
}