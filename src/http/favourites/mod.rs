use crate::http::HttpState;
use axum::Router;

pub mod manage;
pub mod query;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/favourites", axum::routing::get(query::get_favourites))
        .route("/favourites", axum::routing::post(manage::create_favourite))
        .route(
            "/favourites",
            axum::routing::delete(manage::delete_favourite),
        )
        .route(
            "/favourites/groups/{id}",
            axum::routing::get(query::get_favourites_by_group),
        )
        .route(
            "/favourites/groups/{target_type}/{target_id}",
            axum::routing::patch(manage::update_favourite_group),
        )
        .route(
            "/favourites/{target_type}/{target_id}",
            axum::routing::get(query::get_favourite),
        )
}
