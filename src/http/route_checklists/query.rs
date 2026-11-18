use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::auth::quick::check_route_checklist;
use crate::db::resource::checklist::checklist::RouteChecklists;
use crate::db::resource::checklist::checklist_def::RouteChecklist;
use crate::db::resource::checklist::checklist_item::RouteChecklistItems;
use crate::db::resource::checklist::checklist_item_def::RouteChecklistItem;
use crate::extractors::authenticator::AuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[utoipa::path(
    get,
    path = "/route-checklists/{id}",
    tag = "Route Checklists",
    summary = "Get a route checklist",
    params(
        ("id" = i64, Path, description = "Route checklist ID")
    ),
    responses(
        (status = 200, description = "Route checklist", body = RouteChecklist),
        (status = 401, description = "You do not own this checklist"),
        (status = 404, description = "Checklist not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn get_route_checklist(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser
) -> Result<Response, ApiError> {
    let item = check_route_checklist(state.db.clone(), id, user_id).await?;

    let filled_out = RouteChecklists::fill_out(state.db.clone(), vec![item])
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(filled_out[0].clone())).into_response())
}

#[utoipa::path(
    get,
    path = "/route-checklists/{id}/items",
    tag = "Route Checklists",
    summary = "Get items in a route checklist",
    params(
        ("id" = i64, Path, description = "Route checklist ID")
    ),
    responses(
        (status = 200, description = "Route checklist items", body = Vec<RouteChecklistItem>),
        (status = 401, description = "You do not own this checklist"),
        (status = 404, description = "Checklist not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn get_route_checklist_items(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    check_route_checklist(state.db.clone(), id, user_id).await?;

    let items = state
        .db
        .route_checklist_items
        .get_by_checklist(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = RouteChecklistItems::fill_out(state.db.clone(), items)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(filled_out)).into_response())
}

#[utoipa::path(
    get,
    path = "/route-checklists",
    tag = "Route Checklists",
    summary = "Get the user's route checklists",
    responses(
        (status = 200, description = "Route checklists", body = [RouteChecklist]),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn get_route_checklists(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser
) -> Result<Response, ApiError> {
    let items = state
        .db
        .route_checklists
        .get_by_owner(user_id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = RouteChecklists::fill_out(state.db.clone(), items)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(filled_out)).into_response())
}