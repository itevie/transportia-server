use crate::http::HttpState;
use axum::Router;
use axum::routing::get;

mod tram_announcement_audios;
pub mod tram_liveries;
pub mod tram_operators;
pub mod tram_types;
pub mod vehicle;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/trams",
            get(vehicle::search_trams).post(vehicle::create_tram),
        )
        .route(
            "/trams/{id}",
            get(vehicle::get_tram)
                .patch(vehicle::update_tram)
                .delete(vehicle::delete_tram),
        )
        .route(
            "/tram-types",
            get(tram_types::search_tram_types).post(tram_types::create_tram_type),
        )
        .route(
            "/tram-types/{id}",
            get(tram_types::get_tram_type)
                .patch(tram_types::update_tram_type)
                .delete(tram_types::delete_tram_type),
        )
        .route(
            "/tram-operators",
            get(tram_operators::search_tram_operators).post(tram_operators::create_tram_operator),
        )
        .route(
            "/tram-operators/{id}",
            get(tram_operators::get_tram_operator)
                .patch(tram_operators::update_tram_operator)
                .delete(tram_operators::delete_tram_operator),
        )
        .route(
            "/tram-announcement-audios/groups",
            get(tram_announcement_audios::get_tram_announcement_audio_groups),
        )
        .route(
            "/tram-announcement-audios",
            get(tram_announcement_audios::search_tram_announcement_audios),
        )
        .route(
            "/tram-announcement-audios/{id}",
            get(tram_announcement_audios::get_tram_announcement_audio)
                .patch(tram_announcement_audios::update_tram_announcement_audio),
        )
        .route(
            "/tram-announcement-audios/{id}/file",
            get(tram_announcement_audios::get_tram_announcement_audio_file),
        )
        .route(
            "/tram-liveries",
            get(tram_liveries::search_tram_liveries).post(tram_liveries::create_tram_livery),
        )
        .route(
            "/tram-liveries/{id}",
            get(tram_liveries::get_tram_livery)
                .patch(tram_liveries::update_tram_livery)
                .delete(tram_liveries::delete_tram_livery),
        )
}
