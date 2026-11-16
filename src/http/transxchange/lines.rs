use axum::extract::{Query, State};
use axum::Json;
use crate::db;
use crate::db::tx_lines::TxLineQuery;
use crate::http::HttpState;

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
            body = [db::resources::tx_line::TxLine]
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
) -> Result<Json<Vec<db::resources::tx_line::TxLine>>, String> {
    let listings = state
        .db
        .tx_lines
        .search(&query)
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}