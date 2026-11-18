pub mod manage;
pub mod query;

use crate::http::HttpState;
use axum::Router;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/favourites/groups",
            axum::routing::get(query::fetch_favourite_groups),
        )
        .route(
            "/favourites/groups",
            axum::routing::post(manage::create_favourite_group),
        )
        .route(
            "/favourites/groups/{id}",
            axum::routing::patch(manage::update_favourite_group),
        )
        .route(
            "/favourites/groups/{id}",
            axum::routing::delete(manage::delete_favourite_group),
        )
}
