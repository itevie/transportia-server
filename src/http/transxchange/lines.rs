use crate::db;
use crate::db::resource::transxchange::line::TxLineQuery;
use crate::db::resource::transxchange::line_def::TxLine;
use crate::http::HttpState;
use axum::Json;
use axum::extract::{Path, Query, State};
use crate::db::resource::transxchange::operator_def::TxOperator;
use crate::extractors::ratelimit::{QueryRatelimit, RateLimit, SearchRatelimit};

#[utoipa::path(
    get,
    path = "/lines",
    tag = "Transxchange",
    summary = "Search service lines",
    params(TxLineQuery),
    responses(
        (
            status = 200,
            description = "Lines matching the search query",
            body = [TxLine]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_lines(
    State(state): State<HttpState>,
    Query(query): Query<TxLineQuery>,
    _ratelimit: RateLimit<SearchRatelimit>,
) -> Result<Json<Vec<TxLine>>, String> {
    let listings = state
        .db
        .tx_lines
        .search(&query)
        .await
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}



#[utoipa::path(
    get,
    path = "/lines/{id}",
    tag = "Transxchange",
    summary = "Get operator",
    params(
        ("id" = String, Path, description = "Line ID")
    ),
    responses(
        (
            status = 200,
            description = "Line",
            body = TxLine
        ),
        (
            status = 404,
            description = "Line not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_line(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    _ratelimit: RateLimit<QueryRatelimit>,
) -> Result<Json<TxLine>, String> {
    let line = state
        .db
        .tx_lines
        .get(&id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Line not found".to_string())?;

    Ok(Json(line))
}
