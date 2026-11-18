
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::db::big_search::{big_search, BigSearchResult};
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct BigSearchQuery {
    pub query: String,
}

#[utoipa::path(
    get,
    path = "/big-search",
    tag = "Search",
    summary = "Search across all resources",
    params(BigSearchQuery),
    responses(
        (
            status = 200,
            description = "Search results retrieved successfully",
            body = BigSearchResult
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn big_search_route(
    State(state): State<HttpState>,
    Query(params): Query<BigSearchQuery>,
) -> Result<Json<BigSearchResult>, ApiError> {
    let results = big_search(state.db.clone(), params.query)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(results))
}
