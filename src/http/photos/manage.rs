use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use crate::auth::quick::check_photo;
use crate::db::resource::photo::photo_def::PhotoUpdate;
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUserResult;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[utoipa::path(
    patch,
    path = "/photos/{id}",
    tag = "Photos",
    summary = "Update a photo's details",
    request_body = PhotoUpdate,
    responses(
        (
            status = 204,
            description = "Success",
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
pub async fn update_photo(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(update): Json<PhotoUpdate>,
) -> Result<StatusCode, ApiError> {
    check_photo(state.db.clone(), id, user_id).await?;

    state
        .db
        .photos
        .patch(id, &update)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/photos/{id}",
    tag = "Photos",
    summary = "Delete a photo",
    params(
        ("id" = i64, Path, description = "Photo ID")
    ),
    responses(
        (
            status = 204,
            description = "Photo deleted successfully"
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
pub async fn delete_photo(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser
) -> Result<StatusCode, ApiError> {
    check_photo(state.db.clone(), id, user_id).await?;

    state
        .db
        .photos
        .delete(state.db.clone(), id, user_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}