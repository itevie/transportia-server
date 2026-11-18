use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db;
use crate::db::resource::bustimes::livery_def::BtLivery;
use crate::http::HttpState;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct LiverySearch {
    pub query: String,
}

#[utoipa::path(
    get,
    path = "/liveries",
    tag = "Bustimes",
    summary = "Query liveries",
    params(LiverySearch),
    responses(
        (
            status = 200,
            description = "Liveries matching the search query",
            body = [BtLivery]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_liveries(
    State(state): State<HttpState>,
    Query(query): Query<LiverySearch>,
) -> Result<Json<Vec<BtLivery>>, String> {
    let listings = state
        .db
        .liveries
        .search(&query.query)
        .await
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}

#[utoipa::path(
    get,
    path = "/liveries/{id}",
    tag = "Bustimes",
    summary = "Get Livery",
    params(
        ("id" = i64, Path, description = "Livery ID")
    ),
    responses(
        (
            status = 200,
            description = "Livery",
            body = BtLivery
        ),
        (
            status = 404,
            description = "Livery not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_livery(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
) -> Result<Json<BtLivery>, String> {
    let livery = state
        .db
        .liveries
        .get(id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Livery not found".to_string())?;

    Ok(Json(livery))
}