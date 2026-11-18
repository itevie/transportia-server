use crate::http::HttpState;
use axum::Router;

pub mod liveries;
pub mod vehicle_types;
pub mod vehicles;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/liveries", axum::routing::get(liveries::search_liveries))
        .route("/liveries/{id}", axum::routing::get(liveries::get_livery))
        .route(
            "/vehicle-types",
            axum::routing::get(vehicle_types::search_vehicle_types),
        )
        .route(
            "/vehicle-types/{id}",
            axum::routing::get(vehicle_types::get_vehicle_type),
        )
        .route("/vehicles", axum::routing::get(vehicles::search_vehicles))
        .route("/vehicles/{id}", axum::routing::get(vehicles::get_vehicle))
        .route(
            "/vehicles/{id}",
            axum::routing::patch(vehicles::update_vehicle),
        )
}
