use crate::auth::quick::is_trusted_user;
use crate::db;
use crate::db::resource::bustimes::vehicle::{BtVehicleQuery, BtVehicles};
use crate::db::resource::bustimes::vehicle_def::BtVehicleFilleOut;
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/vehicles",
    tag = "Bustimes",
    summary = "Query vehicles",
    params(BtVehicleQuery),
    responses(
        (
            status = 200,
            description = "Vehicles matching the search query",
            body = [BtVehicleFilleOut]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_vehicles(
    State(state): State<HttpState>,
    Query(query): Query<BtVehicleQuery>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Result<Json<Vec<BtVehicleFilleOut>>, String> {
    let listings = state
        .db
        .vehicles
        .search(&query)
        .await
        .map_err(|err| err.to_string())?;

    let filled_out = BtVehicles::fill_out(state.db.clone(), listings, user.map(|x| x.user_id))
        .await
        .map_err(|err| err.to_string())?;

    Ok(Json(filled_out))
}

#[utoipa::path(
    get,
    path = "/vehicles/{id}",
    tag = "Bustimes",
    summary = "Get vehicle",
    params(
        ("id" = i64, Path, description = "Vehicle ID")
    ),
    responses(
        (
            status = 200,
            description = "Vehicle",
            body = BtVehicleFilleOut
        ),
        (
            status = 404,
            description = "Vehicle not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_vehicle(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Result<Json<BtVehicleFilleOut>, String> {
    let vehicle = state
        .db
        .vehicles
        .get(id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Vehicle not found".to_string())?;

    let filled_out = BtVehicles::fill_out(state.db.clone(), vec![vehicle], user.map(|x| x.user_id))
        .await
        .map_err(|err| err.to_string())?
        .into_iter()
        .next()
        .ok_or_else(|| "Vehicle not found".to_string())?;

    Ok(Json(filled_out))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BtVehicleUpdateQuery {
    pub training_vehicle: Option<bool>,
    pub demonstrator: Option<bool>,
    pub open_top: Option<bool>,
}

#[utoipa::path(
    patch,
    path = "/vehicles/{id}",
    tag = "Bustimes",
    summary = "Update a vehicle",
    params(
        ("id" = i64, Path, description = "Vehicle ID")
    ),
    request_body = BtVehicleUpdateQuery,
    responses(
        (
            status = 204,
            description = "Vehicle updated successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 404,
            description = "Vehicle not found"
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
pub async fn update_vehicle(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<BtVehicleUpdateQuery>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut vehicle = state
        .db
        .vehicles
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Vehicle not found".to_string()))?;

    if let Some(training_vehicle) = body.training_vehicle {
        vehicle.training_vehicle = training_vehicle;
    }

    if let Some(demonstrator) = body.demonstrator {
        vehicle.demonstrator = demonstrator;
    }

    if let Some(open_top) = body.open_top {
        vehicle.open_top = open_top;
    }

    state
        .db
        .vehicles
        .upsert(&vehicle)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
