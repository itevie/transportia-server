pub mod listing_fetchers;
pub mod db;
mod http;

use crate::db::bus_listing::BusListingQuery;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use crate::http::HttpState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = match db::init_db() {
        Ok(db) => db,
        Err(err) => {
            println!("{:?}", err);
            return Ok(());
        }
    };

    // let mbc = listing_fetchers::mbc::fetch().await?;
    //
    // for v in mbc {
    //     db.bus_listings.upsert(&v)?;
    // }

    let state = HttpState {
        db: Arc::new(db),
    };

    let app = axum::Router::new()
        .merge(http::routes())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

