use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::db::resource::photo::model_bus_photo_details_def::ModelBusPhotoDetailsUpdate;
use crate::db::resource::photo::photo::Photos;
use crate::extractors::ratelimit::{RateLimit, UpdateRatelimit};
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[utoipa::path(
    patch,
    path = "/photos/{id}/model-bus-details",
    tag = "Photos",
    summary = "Update the model bus details for a photo",
    request_body = ModelBusPhotoDetailsUpdate,
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
    _ratelimit: RateLimit<UpdateRatelimit>,
    Json(update): Json<ModelBusPhotoDetailsUpdate>,
) -> Result<StatusCode, ApiError> {
    // TODO: Require auth and check if user owns it
    let photo = state
        .db
        .photos
        .get(id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Photo not found".to_string()))?;

    let model_bus_details = Photos::get_model_bus_details(state.db.clone(), photo)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    state
        .db
        .model_bus_photo_details
        .update(model_bus_details.id, &update)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
