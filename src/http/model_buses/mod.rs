use crate::http::HttpState;
use crate::http::photo_ratings::{manage, query};
use axum::Router;
use axum::routing::get;

pub mod model_bus_ownerships;
pub mod resource;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/model-buses",
            get(resource::search_model_buses).post(resource::create_model_bus),
        )
        .route(
            "/model-buses/{id}",
            get(resource::get_model_bus)
                .patch(resource::update_model_bus)
                .delete(resource::delete_model_bus),
        )
        .route(
            "/model-buses/possible-matches/{id}",
            get(resource::search_potential_model_buses_for_vehicle),
        )
        .route(
            "/users/{id}/model-bus-ownerships",
            get(model_bus_ownerships::search_model_bus_ownerships)
                .post(model_bus_ownerships::create_model_bus_ownership),
        )
        .route(
            "/users/{id}/model-bus-ownerships/{id}",
            get(model_bus_ownerships::get_model_bus_ownership)
                .patch(model_bus_ownerships::update_model_bus_ownership)
                .delete(model_bus_ownerships::delete_model_bus_ownership),
        )
}
