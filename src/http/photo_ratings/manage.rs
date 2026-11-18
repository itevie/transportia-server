use crate::db::resource::star_rating::star_rating_def::PhotoRating;
use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct PhotoRatingQuery {
    /// Rating from 1 to 5
    pub rating: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PhotoRatingResponse {
    pub rating: Option<i64>,
    pub average_rating: f64,
}

#[utoipa::path(
    post,
    path = "/photos/{id}/ratings",
    tag = "Photo Ratings",
    summary = "Rate a photo",
    params(
        (
            "id" = i64,
            Path,
            description = "Photo ID"
        ),
        PhotoRatingQuery
    ),
    responses(
        (
            status = 200,
            description = "Photo rated successfully",
            body = PhotoRatingResponse
        ),
        (
            status = 400,
            description = "Rating must be between 1 and 5"
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
pub async fn rate_photo(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(photo_id): Path<i64>,
    Query(query): Query<PhotoRatingQuery>,
) -> Result<Json<PhotoRatingResponse>, ApiError> {
    if !(1..=5).contains(&query.rating) {
        return Err(ApiError::BadRequest(
            "Rating must be between 1 and 5".to_string(),
        ));
    }

    let existing = state
        .db
        .photo_ratings
        .get_by_user_and_photo(user_id, photo_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let rating = PhotoRating {
        id: existing.map(|rating| rating.id).unwrap_or(0),
        photo_id,
        user_id,
        rating: query.rating,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    if rating.id == 0 {
        state
            .db
            .photo_ratings
            .create(&rating)
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;
    } else {
        state
            .db
            .photo_ratings
            .upsert(&rating)
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;
    }

    let average_rating = state
        .db
        .photo_ratings
        .average_rating(photo_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(PhotoRatingResponse {
        rating: Some(query.rating),
        average_rating,
    }))
}

#[utoipa::path(
    delete,
    path = "/photos/{id}/ratings",
    tag = "Photo Ratings",
    summary = "Remove your rating from a photo",
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
            description = "Photo rating deleted successfully",
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
pub async fn delete_photo_rating(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(photo_id): Path<i64>,
) -> Result<Json<PhotoRatingResponse>, ApiError> {
    state
        .db
        .photo_ratings
        .delete_by_user_and_photo(user_id, photo_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let average_rating = state
        .db
        .photo_ratings
        .average_rating(photo_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(PhotoRatingResponse {
        rating: None,
        average_rating,
    }))
}
