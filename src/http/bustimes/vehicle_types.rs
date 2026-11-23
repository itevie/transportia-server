use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::resource::bustimes::vehicle_type::VehicleTypeQuery;
use crate::extractors::ratelimit::{QueryRatelimit, RateLimit, SearchRatelimit};

#[utoipa::path(
    get,
    path = "/vehicle-types",
    tag = "Bustimes",
    summary = "Query vehicle types",
    params(VehicleTypeQuery),
    responses(
        (
            status = 200,
            description = "Vehicle types matching the search query",
            body = [BtVehicleType]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_vehicle_types(
    State(state): State<HttpState>,
    Query(query): Query<VehicleTypeQuery>,
    _ratelimit: RateLimit<SearchRatelimit>
) -> Result<Json<Vec<BtVehicleType>>, ApiError> {
    let listings = state
        .db
        .vehicle_types
        .search(&query)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(listings))
}

#[utoipa::path(
    get,
    path = "/vehicles-types/{id}",
    tag = "Bustimes",
    summary = "Get vehicle type",
    params(
        ("id" = i64, Path, description = "Vehicle type ID")
    ),
    responses(
        (
            status = 200,
            description = "Vehicle Type",
            body = BtVehicleType
        ),
        (
            status = 404,
            description = "Vehicle type not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_vehicle_type(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
) -> Result<Json<BtVehicleType>, String> {
    let vehicle = state
        .db
        .vehicle_types
        .get(id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Vehicle type not found".to_string())?;

    Ok(Json(vehicle))
}
