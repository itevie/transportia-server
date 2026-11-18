use crate::db::resource::model_bus_listings::model_bus_listing::{
    BusListingQuery, BusListingQueryOptions,
};
use crate::db::resource::model_bus_listings::model_bus_listing_def::BusListingResource;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};

pub async fn get_listings(
    State(state): State<HttpState>,
    Query(query): Query<BusListingQuery>,
) -> Result<Json<Vec<BusListingResource>>, String> {
    let listings = state
        .db
        .bus_listings
        .search(&query)
        .await
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}

pub async fn get_listing_options(
    State(state): State<HttpState>,
) -> Result<Json<BusListingQueryOptions>, String> {
    let options = state
        .db
        .bus_listings
        .get_query_options()
        .await
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
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(model_buses))
}
