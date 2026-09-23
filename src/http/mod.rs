use std::sync::Arc;
use axum::Router;
use crate::db::Database;

pub mod bus_listings;

#[derive(Clone)]
pub struct HttpState {
    pub db: Arc<Database>,
}

pub fn routes() -> Router<HttpState> {
    Router::new()
        .merge(bus_listings::routes())
}