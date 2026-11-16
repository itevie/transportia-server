use axum::Router;
use crate::http::bus_listings::query;
use crate::http::HttpState;

pub mod siri;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/siri", axum::routing::get(siri::get_vehicle_monitoring))
}