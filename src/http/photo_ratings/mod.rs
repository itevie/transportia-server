use crate::http::HttpState;
use crate::http::photos::{bus_details, upload};
use axum::Router;
use axum::extract::DefaultBodyLimit;

pub mod manage;
pub mod query;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/photos/{id}/ratings",
            axum::routing::post(manage::rate_photo),
        )
        .route(
            "/photos/{id}/ratings",
            axum::routing::delete(manage::delete_photo_rating),
        )
        .route(
            "/photos/{id}/ratings",
            axum::routing::get(query::get_photo_rating),
        )
}
