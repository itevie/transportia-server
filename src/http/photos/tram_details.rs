
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db::photo::Photos;
use crate::db::resources::tram_photo_details::TramPhotoDetailsUpdate;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[utoipa::path(
    patch,
    path = "/photos/{id}/tram-details",
    tag = "Photos",
    summary = "Update the tram details for a photo",
    request_body = TramPhotoDetailsUpdate,
    responses(
        (
            status = 204,
            description = "Success"
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
pub async fn update_details(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    Json(update): Json<TramPhotoDetailsUpdate>,
) -> Result<StatusCode, ApiError> {
    // TODO: Require auth and check if user owns it
    let photo = state
        .db
        .photos
        .get(id)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Photo not found".to_string()))?;

    let tram_details = Photos::get_tram_details(state.db.clone(), photo)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    state
        .db
        .tram_photo_details
        .update(tram_details.id, &update)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
