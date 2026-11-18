use crate::http::HttpState;
use axum::Router;

pub mod login;
pub mod query;
pub mod registration;
pub mod validation;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/register", axum::routing::post(registration::register))
        .route("/users/{id}", axum::routing::get(query::get_user))
        .route("/login", axum::routing::post(login::login))
        .route("/me", axum::routing::get(query::me))
        .route("/users", axum::routing::get(query::search_users))
}
