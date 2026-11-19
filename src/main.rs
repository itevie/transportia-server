mod auth;
mod bods;
pub mod db;
mod extractors;
pub mod fetchers;
pub mod http;
pub mod util;
mod validation;

use crate::bods::datafeed::BodsSiriVmClient;
use crate::db::create_missing_comment_threads;
use crate::db::resource::tram::tram_announcement_audio::TramAnnouncementAudios;
use crate::http::{ApiDoc, HttpState};
use axum::extract::DefaultBodyLimit;
use axum::http::{header, Method};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=trace")
        .with_target(true)
        .init();

    let db = match db::init_db().await {
        Ok(db) => db,
        Err(err) => {
            println!("Error init db: {:?}", err);
            return Ok(());
        }
    };

    let db_arc = Arc::new(db);

    match TramAnnouncementAudios::scan_tram_audios(db_arc.clone(), "./data/tram_audios").await {
        Ok(_) => println!("Scanned tram announcements!"),
        Err(e) => eprintln!("Failed to scan tram announcements: {}", e.to_string()),
    };

    println!("Creating comment threads");
    create_missing_comment_threads(db_arc.clone(), "tx_operators").await?;
    create_missing_comment_threads(db_arc.clone(), "bt_vehicles").await?;
    create_missing_comment_threads(db_arc.clone(), "photos").await?;
    // create_missing_comment_threads(db_arc.clone(), "tx_routes").await?;
    println!("Created comment threads");

    fetchers::run_all_in_background(db_arc.clone());

    // let mbc = fetchers::mbc::fetch().await?;
    //
    // for v in mbc {
    //     db_arc.bus_listings.upsert(&v)?;
    // }

    let state = HttpState {
        db: db_arc.clone(),
        s3: Arc::from(util::s3::create_r2_client().await),
        bods_client: BodsSiriVmClient::new(std::env::var("BODS_API_KEY").unwrap()).unwrap(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = axum::Router::new()
        .merge(http::routes())
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", ApiDoc::openapi()),
        )
        .layer(cors)
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
