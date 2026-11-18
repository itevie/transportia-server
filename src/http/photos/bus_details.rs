use crate::db::resource::photo::bus_photo_details_def::BusPhotoDetailsUpdate;
use crate::db::resource::photo::photo::Photos;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

#[utoipa::path(
    patch,
    path = "/photos/{id}/bus-details",
    tag = "Photos",
    summary = "Update the bus details for a photo",
    request_body = BusPhotoDetailsUpdate,
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
    Json(update): Json<BusPhotoDetailsUpdate>,
) -> Result<StatusCode, ApiError> {
    // TODO: Require auth and check if user owns it
    let photo = state
        .db
        .photos
        .get(id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Photo not found".to_string()))?;

    let bus_details = Photos::get_bus_details(state.db.clone(), photo)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    state
        .db
        .bus_photo_details
        .update(bus_details.id, &update)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
