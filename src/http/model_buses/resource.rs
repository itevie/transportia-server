use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resource::model_bus::model_bus::{ModelBusQuery, ModelBuses};
use crate::db::resource::model_bus::model_bus_def::{ModelBus, ModelBusFilledOut};
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ModelBusCreateRequest {
    pub product_code: String,
    pub manufacturer: String,
    pub scale: i64,

    pub operator: Option<String>,
    pub operator_link: Option<String>,

    pub livery: Option<String>,
    pub livery_link: Option<i64>,

    pub fleet_number: Option<String>,
    pub registration: Option<String>,
    pub vehicle_link: Option<i64>,

    pub route_number: Option<String>,
    pub destination: Option<String>,
    pub route_link: Option<String>,

    pub vehicle_type: Option<String>,
    pub vehicle_type_link: Option<i64>,

    pub has_certificate: bool,
    pub certificate_max: i64,

    pub released: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ModelBusUpdateRequest {
    pub product_code: Option<String>,
    pub manufacturer: Option<String>,
    pub scale: Option<i64>,

    pub operator: Option<String>,
    pub operator_link: Option<String>,

    pub livery: Option<String>,
    pub livery_link: Option<i64>,

    pub fleet_number: Option<String>,
    pub registration: Option<String>,
    pub vehicle_link: Option<i64>,

    pub route_number: Option<String>,
    pub destination: Option<String>,
    pub route_link: Option<String>,

    pub vehicle_type: Option<String>,
    pub vehicle_type_link: Option<i64>,

    pub has_certificate: Option<bool>,
    pub certificate_max: Option<i64>,

    pub released: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModelBusCreateResponse {
    pub id: i64,
}

#[utoipa::path(
    get,
    path = "/model-buses",
    tag = "Model Buses",
    summary = "Search model buses",
    params(ModelBusQuery),
    responses(
        (
            status = 200,
            description = "Model buses matching the search query",
            body = [ModelBusFilledOut]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_model_buses(
    State(state): State<HttpState>,
    Query(query): Query<ModelBusQuery>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<ModelBusFilledOut>>, ApiError> {
    let model_buses = state
        .db
        .model_buses
        .search(&query)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = ModelBuses::fill_out(state.db.clone(), model_buses, None)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out))
}

#[utoipa::path(
    get,
    path = "/model-buses/potential-matches/{id}",
    tag = "Model Buses",
    summary = "Search model buses",
    params(
        ("id" = i64, Path, description = "Model bus ID")
    ),
    responses(
        (
            status = 200,
            description = "Model buses matching the search query",
            body = [ModelBusFilledOut]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_potential_model_buses_for_vehicle(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<ModelBusFilledOut>>, ApiError> {
    let model_buses = state
        .db
        .model_buses
        .search(&ModelBusQuery {
            search: Some(id),
            product_code: None,
            manufacturer: None,
            scale: None,
            operator: None,
            operator_link: None,
            livery: None,
            livery_link: None,
            fleet_number: None,
            registration: None,
            vehicle_link: None,
            route_number: None,
            destination: None,
            route_link: None,
            vehicle_type: None,
            vehicle_type_link: None,
            has_certificate: None,
            released: None,
            has_image: None,
            page_length: None,
            offset: None,
        })
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = ModelBuses::fill_out(state.db.clone(), model_buses, None)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out))
}

#[utoipa::path(
    get,
    path = "/model-buses/{id}",
    tag = "Model Buses",
    summary = "Get model bus",
    params(
        ("id" = i64, Path, description = "Model bus ID")
    ),
    responses(
        (
            status = 200,
            description = "Model bus",
            body = ModelBusFilledOut
        ),
        (
            status = 404,
            description = "Model bus not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_model_bus(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<ModelBusFilledOut>, ApiError> {
    let model_bus = state
        .db
        .model_buses
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Model bus not found".to_string()))?;

    let filled_out = ModelBuses::fill_out(state.db.clone(), vec![model_bus], None)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out.get(0).unwrap().clone()))
}

#[utoipa::path(
    post,
    path = "/model-buses",
    tag = "Model Buses",
    summary = "Create a model bus",
    request_body = ModelBusCreateRequest,
    responses(
        (
            status = 201,
            description = "Model bus created successfully",
            body = ModelBusCreateResponse
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
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_model_bus(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<ModelBusCreateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let now = chrono::Utc::now().to_rfc3339();

    let model_bus = ModelBus {
        id: 0,

        product_code: body.product_code,
        manufacturer: body.manufacturer,
        scale: body.scale,

        operator: body.operator,
        operator_link: body.operator_link,

        livery: body.livery,
        livery_link: body.livery_link,

        fleet_number: body.fleet_number,
        registration: body.registration,
        vehicle_link: body.vehicle_link,

        route_number: body.route_number,
        destination: body.destination,
        route_link: body.route_link,

        vehicle_type: body.vehicle_type,
        vehicle_type_link: body.vehicle_type_link,

        has_certificate: body.has_certificate,
        certificate_max: body.certificate_max,

        released: body.released,

        created_at: now.clone(),
        updated_at: now,
    };

    let id = state
        .db
        .model_buses
        .create(&model_bus)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::CREATED, Json(ModelBusCreateResponse { id })).into_response())
}

#[utoipa::path(
    patch,
    path = "/model-buses/{id}",
    tag = "Model Buses",
    summary = "Update a model bus",
    params(
        ("id" = i64, Path, description = "Model bus ID")
    ),
    request_body = ModelBusUpdateRequest,
    responses(
        (
            status = 204,
            description = "Model bus updated successfully"
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
            description = "Model bus not found"
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
pub async fn update_model_bus(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<ModelBusUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut model_bus = state
        .db
        .model_buses
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Model bus not found".to_string()))?;

    if let Some(product_code) = body.product_code {
        model_bus.product_code = product_code;
    }

    if let Some(manufacturer) = body.manufacturer {
        model_bus.manufacturer = manufacturer;
    }

    if let Some(scale) = body.scale {
        model_bus.scale = scale;
    }

    if let Some(operator) = body.operator {
        model_bus.operator = Some(operator);
    }

    if let Some(operator_link) = body.operator_link {
        model_bus.operator_link = Some(operator_link);
    }

    if let Some(livery) = body.livery {
        model_bus.livery = Some(livery);
    }

    if let Some(livery_link) = body.livery_link {
        model_bus.livery_link = Some(livery_link);
    }

    if let Some(fleet_number) = body.fleet_number {
        model_bus.fleet_number = Some(fleet_number);
    }

    if let Some(registration) = body.registration {
        model_bus.registration = Some(registration);
    }

    if let Some(vehicle_link) = body.vehicle_link {
        model_bus.vehicle_link = Some(vehicle_link);
    }

    if let Some(route_number) = body.route_number {
        model_bus.route_number = Some(route_number);
    }

    if let Some(destination) = body.destination {
        model_bus.destination = Some(destination);
    }

    if let Some(route_link) = body.route_link {
        model_bus.route_link = Some(route_link);
    }

    if let Some(vehicle_type) = body.vehicle_type {
        model_bus.vehicle_type = Some(vehicle_type);
    }

    if let Some(vehicle_type_link) = body.vehicle_type_link {
        model_bus.vehicle_type_link = Some(vehicle_type_link);
    }

    if let Some(has_certificate) = body.has_certificate {
        model_bus.has_certificate = has_certificate;
    }

    if let Some(certificate_max) = body.certificate_max {
        model_bus.certificate_max = certificate_max;
    }

    if let Some(released) = body.released {
        model_bus.released = Some(released);
    }

    model_bus.updated_at = chrono::Utc::now().to_rfc3339();

    state
        .db
        .model_buses
        .upsert(&model_bus)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    delete,
    path = "/model-buses/{id}",
    tag = "Model Buses",
    summary = "Delete a model bus",
    params(
        ("id" = i64, Path, description = "Model bus ID")
    ),
    responses(
        (
            status = 204,
            description = "Model bus deleted successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not an admin"
        ),
        (
            status = 404,
            description = "Model bus not found"
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
pub async fn delete_model_bus(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    is_admin(state.db.clone(), user_id).await?;

    state
        .db
        .model_buses
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Model bus not found".to_string()))?;

    state
        .db
        .model_buses
        .delete(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
