use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::ToSchema;
use crate::auth::quick::check_favourites_group;
use crate::db::resource::favourite::favourite_group_def::FavouritesGroup;
use crate::extractors::authenticator::AuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Deserialize, ToSchema)]
pub struct FavouriteGroupCreate {
    pub favourite_type: i64,
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/favourites-groups",
    tag = "Favourite Groups",
    summary = "Create a favourite group",
    description = "Creates a new favourite group.",
    request_body = FavouriteGroupCreate,
    responses(
        (
            status = 201,
            description = "Favourite group created successfully",
            body = FavouritesGroup
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
pub async fn create_favourite_group(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Json(data): Json<FavouriteGroupCreate>,
) -> Result<Response, ApiError> {
    let id = state
        .db
        .favourites_groups
        .create(
            data.favourite_type,
            user_id,
            &data.name,
            &chrono::Utc::now().to_rfc3339(),
        )
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let group = state
        .db
        .favourites_groups
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| {
            ApiError::Database("Created favourite group could not be found".to_string())
        })?;

    Ok((StatusCode::CREATED, Json(group)).into_response())
}

#[derive(Deserialize, ToSchema)]
pub struct FavouriteGroupUpdate {
    pub favourite_type: i64,
    pub name: String,
}

#[utoipa::path(
    patch,
    path = "/favourites-groups/{id}",
    tag = "Favourite Groups",
    summary = "Update a favourite group",
    description = "Updates an existing favourite group's type and name.",
    params(
        ("id" = i64, Path, description = "Favourite group ID")
    ),
    request_body = FavouriteGroupUpdate,
    responses(
        (
            status = 200,
            description = "Favourite group updated successfully",
            body = FavouritesGroup
        ),
        (
            status = 404,
            description = "Favourite group not found"
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
pub async fn update_favourite_group(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
    Json(data): Json<FavouriteGroupUpdate>,
) -> Result<Response, ApiError> {
    let mut group = check_favourites_group(state.db.clone(), id, user_id).await?;

    group.favourite_type = data.favourite_type;
    group.name = data.name;

    state
        .db
        .favourites_groups
        .update(&group)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(group)).into_response())
}

#[utoipa::path(
    delete,
    path = "/favourites-groups/{id}",
    tag = "Favourite Groups",
    summary = "Delete a favourite group",
    description = "Deletes a favourite group by its ID.",
    params(
        ("id" = i64, Path, description = "Favourite group ID")
    ),
    responses(
        (
            status = 204,
            description = "Favourite group deleted successfully"
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
pub async fn delete_favourite_group(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    check_favourites_group(state.db.clone(), id, user_id).await?;
    state
        .db
        .favourites_groups
        .delete(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}