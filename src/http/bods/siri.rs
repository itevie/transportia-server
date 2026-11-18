use crate::bods::datafeed::{BodsSiriVmClient, DatafeedQuery, VehicleJourney};
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::{
    Json,
    extract::{Query, State},
};
use utoipa::path;

#[utoipa::path(
    get,
    path = "/bods/vehicle-monitoring",
    tag = "BODS",
    summary = "Fetch live bus vehicle journeys",
    params(DatafeedQuery),
    responses(
        (
            status = 200,
            description = "Live vehicle journeys retrieved successfully",
            content_type = "application/json",
            body = [VehicleJourney]
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
pub async fn get_vehicle_monitoring(
    Query(query): Query<DatafeedQuery>,
    State(state): State<HttpState>,
) -> Result<Json<Vec<VehicleJourney>>, ApiError> {
    let data = state
        .bods_client
        .get_datafeed(&query)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let journeys = BodsSiriVmClient::convert(data);

    Ok(Json(journeys))
}
