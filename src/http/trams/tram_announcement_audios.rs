use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resource::tram::tram_announcement_audio::TramAnnouncementAudioQuery;
use crate::db::resource::tram::tram_announcement_audio_def::TramAnnouncementAudio;
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TramAnnouncementAudioSearchQuery {
    pub file_name: Option<String>,
    pub audio_group: Option<String>,
    pub label: Option<String>,
    pub operator: Option<i64>,
    pub route: Option<i64>,
    pub next_stop: Option<i64>,
    pub search: Option<String>,
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramAnnouncementAudioUpdateRequest {
    pub label: Option<String>,
    pub operator_id: Option<i64>,
    pub route_id: Option<i64>,
    pub next_stop_id: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TramAnnouncementAudioCreateResponse {
    pub id: i64,
}

#[utoipa::path(
    get,
    path = "/tram-announcement-audios",
    tag = "Trams",
    summary = "Search tram announcement audios",
    params(TramAnnouncementAudioSearchQuery),
    responses(
        (
            status = 200,
            description = "Tram announcement audios matching the search query",
            body = [TramAnnouncementAudio]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_tram_announcement_audios(
    State(state): State<HttpState>,
    Query(query): Query<TramAnnouncementAudioSearchQuery>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<TramAnnouncementAudio>>, ApiError> {
    let audios = state
        .db
        .tram_announcement_audios
        .search(&TramAnnouncementAudioQuery {
            file_name: query.file_name,
            audio_group: query.audio_group,
            label: query.label,
            operator: query.operator,
            route: query.route,
            next_stop: query.next_stop,
            search: query.search,
            page_length: query.page_length,
            offset: query.offset,
        })
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(audios))
}

#[utoipa::path(
    get,
    path = "/tram-announcement-audios/{id}",
    tag = "Trams",
    summary = "Get tram announcement audio",
    params(
        ("id" = i64, Path, description = "Tram announcement audio ID")
    ),
    responses(
        (
            status = 200,
            description = "Tram announcement audio",
            body = TramAnnouncementAudio
        ),
        (
            status = 404,
            description = "Tram announcement audio not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_tram_announcement_audio(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<TramAnnouncementAudio>, ApiError> {
    let audio = state
        .db
        .tram_announcement_audios
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram announcement audio not found".to_string()))?;

    Ok(Json(audio))
}

#[utoipa::path(
    get,
    path = "/tram-announcement-audios/groups",
    tag = "Tram Announcement Audios",
    responses(
        (status = 200, description = "Successfully retrieved audio groups", body = Vec<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_tram_announcement_audio_groups(
    State(state): State<HttpState>,
) -> Result<impl IntoResponse, ApiError> {
    let groups = state
        .db
        .tram_announcement_audios
        .get_groups()
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(groups))
}

#[utoipa::path(
    patch,
    path = "/tram-announcement-audios/{id}",
    tag = "Trams",
    summary = "Update a tram announcement audio",
    params(
        ("id" = i64, Path, description = "Tram announcement audio ID")
    ),
    request_body = TramAnnouncementAudioUpdateRequest,
    responses(
        (
            status = 204,
            description = "Tram announcement audio updated successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 404,
            description = "Tram announcement audio not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_tram_announcement_audio(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramAnnouncementAudioUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut audio = state
        .db
        .tram_announcement_audios
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram announcement audio not found".to_string()))?;

    if let Some(label) = body.label {
        audio.label = Some(label);
    }

    if let Some(operator_id) = body.operator_id {
        audio.operator_id = Some(operator_id);
    }

    if let Some(route_id) = body.route_id {
        audio.route_id = Some(route_id);
    }

    if let Some(next_stop_id) = body.next_stop_id {
        audio.next_stop_id = Some(next_stop_id);
    }

    state
        .db
        .tram_announcement_audios
        .update(&audio)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    get,
    path = "/tram-announcement-audios/{id}/file",
    tag = "Trams",
    summary = "Get tram announcement audio file",
    params(
        ("id" = i64, Path, description = "Tram announcement audio ID")
    ),
    responses(
        (
            status = 200,
            description = "Tram announcement audio file",
            content_type = "audio/mpeg",
            body = String,
            example = "Binary audio data"
        ),
        (
            status = 404,
            description = "Tram announcement audio or file not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_tram_announcement_audio_file(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Response, ApiError> {
    let audio = state
        .db
        .tram_announcement_audios
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram announcement audio not found".to_string()))?;

    let data_dir = tokio::fs::canonicalize("data/tram_audios")
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let file_path = data_dir.join(&audio.audio_group).join(&audio.file_name);

    let file_path = tokio::fs::canonicalize(file_path).await.map_err(|err| {
        println!("Canonicalize error: {err}");
        ApiError::NotFound("Tram announcement audio file not found".to_string())
    })?;

    if !file_path.starts_with(&data_dir) {
        return Err(ApiError::NotFound(
            "Tram announcement audio file not found".to_string(),
        ));
    }

    let contents = tokio::fs::read(file_path)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(([(axum::http::header::CONTENT_TYPE, "audio/mpeg")], contents).into_response())
}
