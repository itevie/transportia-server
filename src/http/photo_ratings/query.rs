use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use crate::http::photo_ratings::manage::PhotoRatingResponse;
use axum::Json;
use axum::extract::{Path, State};
use crate::extractors::ratelimit::{RateLimit, RatingRatelimit};

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
    _ratelimit: RateLimit<RatingRatelimit>,
    Path(photo_id): Path<i64>,
) -> Result<Json<PhotoRatingResponse>, ApiError> {
    let rating = state
        .db
        .photo_ratings
        .get_by_user_and_photo(user_id, photo_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let average_rating = state
        .db
        .photo_ratings
        .average_rating(photo_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(PhotoRatingResponse {
        rating: rating.map(|rating| rating.rating),
        average_rating,
    }))
}
