use crate::db;
use crate::db::resource::transxchange::line::TxLineQuery;
use crate::db::resource::transxchange::line_def::TxLine;
use crate::http::HttpState;
use axum::Json;
use axum::extract::{Query, State};

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
) -> Result<Json<Vec<TxLine>>, String> {
    let listings = state
        .db
        .tx_lines
        .search(&query)
        .await
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}
