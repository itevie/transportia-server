use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct OperatorsSearch {
    pub query: String,
}

#[utoipa::path(
    get,
    path = "/operators",
    tag = "Transxchange",
    summary = "Search operators",
    params(OperatorsSearch),
    responses(
        (
            status = 200,
            description = "Operators matching the search query",
            body = [db::resources::tx_operator::TxOperator]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_operators(
    State(state): State<HttpState>,
    Query(query): Query<OperatorsSearch>,
) -> Result<Json<Vec<db::resources::tx_operator::TxOperator>>, ApiError> {
    let operators = state
        .db
        .tx_operators
        .search(&query.query)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(operators))
}

#[utoipa::path(
    get,
    path = "/operators/{id}",
    tag = "Transxchange",
    summary = "Get operator",
    params(
        ("id" = String, Path, description = "Operator ID")
    ),
    responses(
        (
            status = 200,
            description = "Operator",
            body = db::resources::tx_operator::TxOperator
        ),
        (
            status = 404,
            description = "Operator not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_operator(
    State(state): State<HttpState>,
    Path(id): Path<String>,
) -> Result<Json<db::resources::tx_operator::TxOperator>, String> {
    println!("{}", id);
    let operator = state
        .db
        .tx_operators
        .get(&id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "Operator not found".to_string())?;


    Ok(Json(operator))
}