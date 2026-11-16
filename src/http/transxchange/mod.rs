use axum::Router;
use crate::http::HttpState;

pub mod operators;
pub mod lines;
mod trips;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/operators", axum::routing::get(operators::search_operators))
        .route("/operators/{id}", axum::routing::get(operators::get_operator))
        .route("/lines", axum::routing::get(lines::search_lines))
        .route("/trips", axum::routing::get(trips::search_trips))
        .route("/trips/today", axum::routing::get(trips::get_trips_today))
        
}