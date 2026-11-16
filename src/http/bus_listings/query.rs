use axum::extract::{Path, Query, State};
use axum::Json;
use crate::db;
use crate::db::bus_listings::BusListingQuery;
use crate::db::model_buses::{ModelBusQuery, ModelBuses};
use crate::db::resources::bus_listing::BusListingResource;
use crate::db::resources::model_bus::ModelBusFilledOut;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

pub async fn get_listings(
    State(state): State<HttpState>,
    Query(query): Query<db::bus_listings::BusListingQuery>,
) -> Result<Json<Vec<db::resources::bus_listing::BusListingResource>>, String> {
    let listings = state
        .db
        .bus_listings
        .search(&query)
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}

pub async fn get_listing_options(
    State(state): State<HttpState>,
) -> Result<Json<db::bus_listings::BusListingQueryOptions>, String> {
    let options = state
        .db
        .bus_listings
        .get_query_options()
        .map_err(|err| err.to_string())?;

    Ok(Json(options))
}
#[utoipa::path(
    get,
    path = "/litings/potential-matches/{id}",
    summary = "Search potential listings for model buses",
    params(
        ("id" = i64, Path, description = "Query")
    ),
    responses(
        (
            status = 200,
            description = "Listings matching the search query",
            body = [BusListingResource]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_potential_listings_for_model(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<BusListingResource>>, ApiError> {
    let model_buses = state
        .db
        .bus_listings
        .search(&BusListingQuery {
            scale: None,
            origin: None,
            title: None,
            manufacturer: None,
            manufacturer_code: None,
            price_low: None,
            price_high: None,
            condition: None,
            category: None,
            search: Some(id),
            sort: None,
            page_length: None,
            offset: None,
        })
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(model_buses))
}