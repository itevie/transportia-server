use crate::http::HttpState;
use axum::Router;

pub mod query;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/listings", axum::routing::get(query::get_listings))
        .route(
            "/listings/options",
            axum::routing::get(query::get_listing_options),
        )
        .route(
            "/listings/potential-matches/{id}",
            axum::routing::get(query::search_potential_listings_for_model),
        )
}
