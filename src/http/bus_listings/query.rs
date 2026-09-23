use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use crate::db;
use crate::http::HttpState;

pub async fn get_listings(
    State(state): State<HttpState>,
    Query(query): Query<db::bus_listing::BusListingQuery>,
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
) -> Result<Json<db::bus_listing::BusListingQueryOptions>, String> {
    let options = state
        .db
        .bus_listings
        .get_query_options()
        .map_err(|err| err.to_string())?;

    Ok(Json(options))
}