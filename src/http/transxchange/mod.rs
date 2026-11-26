use crate::http::HttpState;
use axum::Router;

pub mod lines;
pub mod operators;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/operators",
            axum::routing::get(operators::search_operators),
        )
        .route(
            "/operators/{id}",
            axum::routing::get(operators::get_operator),
        )
        .route("/lines", axum::routing::get(lines::search_lines))
        .route("/lines/{id}", axum::routing::get(lines::get_line))
}
