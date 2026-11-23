use crate::db;
use crate::db::resource::photo::photo::{PhotoQuery, Photos};
use crate::db::resource::photo::photo_def::PhotoFilledOut;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::extractors::ratelimit::{DeleteRatelimit, QueryRatelimit, RateLimit, SearchRatelimit};

#[utoipa::path(
    get,
    path = "/photos",
    tag = "Photos",
    summary = "Search photos",
    params(PhotoQuery),
    responses(
        (
            status = 200,
            description = "List of photos",
            body = [PhotoFilledOut]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn query_photos(
    State(state): State<HttpState>,
    OptionalAuthUser(user): OptionalAuthUser,
    _ratelimit: RateLimit<SearchRatelimit>,
    Query(query): Query<PhotoQuery>,
) -> Result<Json<Vec<PhotoFilledOut>>, ApiError> {
    let photos = state
        .db
        .photos
        .search(&query)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let filled_out = Photos::fill_out(state.db.clone(), photos, user.map(|x| x.user_id))
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(filled_out))
}

#[utoipa::path(
    get,
    tag = "Photos",
    path = "/photos/{id}",
    summary = "Get a photo by ID",
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
            description = "Photo details",
            body = PhotoFilledOut
        ),
        (
            status = 404,
            description = "Photo not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_photo(
    State(state): State<HttpState>,
    OptionalAuthUser(user): OptionalAuthUser,
    _ratelimit: RateLimit<QueryRatelimit>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let photo = state
        .db
        .photos
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or(ApiError::UserNotFound)?;

    let filled_out = Photos::fill_out(state.db.clone(), vec![photo], user.map(|x| x.user_id))
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok((StatusCode::OK, Json(filled_out[0].clone())).into_response())
}
