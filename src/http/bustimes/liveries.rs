use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db;
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
            body = [db::resources::bt_livery::BtLivery]
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
) -> Result<Json<Vec<db::resources::bt_livery::BtLivery>>, String> {
    let listings = state
        .db
        .liveries
        .search(&query.query)
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
            body = db::resources::bt_livery::BtLivery
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
) -> Result<Json<db::resources::bt_livery::BtLivery>, String> {
    let livery = state
        .db
        .liveries
        .get(id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Livery not found".to_string())?;

    Ok(Json(livery))
}