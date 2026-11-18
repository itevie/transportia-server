use crate::http::HttpState;
use crate::http::bus_listings::query;
use axum::Router;

pub mod siri;

pub fn routes() -> Router<HttpState> {
    Router::new().route("/siri", axum::routing::get(siri::get_vehicle_monitoring))
}
