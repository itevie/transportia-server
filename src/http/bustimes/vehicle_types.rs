use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db;
use crate::db::vehicles::BtVehicles;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct VehicleTypeSearch {
    pub query: String,
}

#[utoipa::path(
    get,
    path = "/vehicle-types",
    tag = "Bustimes",
    summary = "Query vehicle types",
    params(VehicleTypeSearch),
    responses(
        (
            status = 200,
            description = "Vehicle types matching the search query",
            body = [db::resources::bt_vehicle_type::BtVehicleType]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_vehicle_types(
    State(state): State<HttpState>,
    Query(query): Query<VehicleTypeSearch>,
) -> Result<Json<Vec<db::resources::bt_vehicle_type::BtVehicleType>>, ApiError> {
    let listings = state
        .db
        .vehicle_types
        .search(&query.query)
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
            body = db::resources::bt_vehicle_type::BtVehicleType
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
) -> Result<Json<db::resources::bt_vehicle_type::BtVehicleType>, String> {
    let vehicle = state
        .db
        .vehicle_types
        .get(id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Vehicle type not found".to_string())?;


    Ok(Json(vehicle))
}