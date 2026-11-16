use axum::Router;
use crate::http::HttpState;

pub mod query;
pub mod manage;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/comment-threads/{id}", axum::routing::get(query::get_comments))
        .route("/comment-threads/{id}/comments", axum::routing::post(manage::create_comment))
        .route("/comments/{id}", axum::routing::delete(manage::delete_comment))
        .route("/comments/{id}", axum::routing::patch(manage::update_comment))
}