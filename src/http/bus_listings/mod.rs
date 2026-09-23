use std::sync::Arc;
use axum::Router;
use crate::db::Database;
use crate::http::HttpState;

pub mod query;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/listings", axum::routing::get(query::get_listings))
        .route("/listings/options", axum::routing::get(query::get_listing_options))
}