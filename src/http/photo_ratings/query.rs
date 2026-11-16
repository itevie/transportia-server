use axum::extract::{Path, State};
use axum::Json;
use crate::extractors::authenticator::AuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;
use crate::http::photo_ratings::manage::PhotoRatingResponse;

#[utoipa::path(
    get,
    path = "/photos/{id}/ratings",
    tag = "Photo Ratings",
    summary = "Get ratings for a photo",
    params(
        (
            "id" = i64,
            Path,
            description = "Photo ID"
        )
    ),
    responses(
        (
            status = 200,
            description = "Photo rating information",
            body = PhotoRatingResponse
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
pub async fn get_photo_rating(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(photo_id): Path<i64>,
) -> Result<Json<PhotoRatingResponse>, ApiError> {
    let rating = state
        .db
        .photo_ratings
        .get_by_user_and_photo(user_id, photo_id)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let average_rating = state
        .db
        .photo_ratings
        .average_rating(photo_id)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(PhotoRatingResponse {
        rating: rating.map(|rating| rating.rating),
        average_rating,
    }))
}