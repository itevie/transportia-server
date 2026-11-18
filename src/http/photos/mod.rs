use crate::http::HttpState;
use axum::Router;
use axum::extract::DefaultBodyLimit;

pub mod bus_details;
pub mod image;
pub mod manage;
mod model_bus_details;
pub mod query;
mod tram_details;
pub mod upload;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/photos/upload",
            axum::routing::post(upload::upload_image)
                .layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route("/photos/{id}", axum::routing::patch(manage::update_photo))
        .route("/photos/{id}", axum::routing::delete(manage::delete_photo))
        .route("/photos/{id}", axum::routing::get(query::get_photo))
        .route(
            "/photos/{id}/image",
            axum::routing::get(image::get_photo_image),
        )
        .route("/photos", axum::routing::get(query::query_photos))
        .route(
            "/photos/{id}/bus-details",
            axum::routing::patch(bus_details::update_details),
        )
        .route(
            "/photos/{id}/tram-details",
            axum::routing::patch(tram_details::update_details),
        )
        .route(
            "/photos/{id}/model-bus-details",
            axum::routing::patch(model_bus_details::update_details),
        )
}
