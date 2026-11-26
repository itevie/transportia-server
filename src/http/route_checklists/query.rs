use crate::auth::quick::check_route_checklist;
use crate::db::resource::checklist::checklist::Checklists;
use crate::db::resource::checklist::checklist_def::Checklist;
use crate::db::resource::checklist::checklist_item::{ChecklistItemSearch, ChecklistItems};
use crate::db::resource::checklist::checklist_item_def::ChecklistItem;
use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::extractors::ratelimit::{ChecklistRatelimit, RateLimit};

#[utoipa::path(
    get,
    path = "/route-checklists/{id}",
    tag = "Route Checklists",
    summary = "Get a route checklist",
    params(
        ("id" = i64, Path, description = "Route checklist ID")
    ),
    responses(
        (status = 200, description = "Route checklist", body = Checklist),
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
    _ratelimit: RateLimit<ChecklistRatelimit>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    let item = check_route_checklist(state.db.clone(), id, user_id).await?;

    let filled_out = Checklists::fill_out(state.db.clone(), vec![item])
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(filled_out[0].clone())).into_response())
}

#[utoipa::path(
    get,
    path = "/route-checklists/{id}/items",
    tag = "Route Checklists",
    summary = "Search items in a route checklist",
    params(
        ("id" = i64, Path, description = "Route checklist ID"),
        ChecklistItemSearch
    ),
    responses(
        (status = 200, description = "Route checklist items", body = Vec<ChecklistItem>),
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
    Query(query): Query<ChecklistItemSearch>,
    _ratelimit: RateLimit<ChecklistRatelimit>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    let checklist = check_route_checklist(state.db.clone(), id, user_id).await?;

    let items = state
        .db
        .checklist_items
        .search(id, &query)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = ChecklistItems::fill_out(state.db.clone(), checklist, items)
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
        (status = 200, description = "Route checklists", body = [Checklist]),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn get_route_checklists(
    State(state): State<HttpState>,
    _ratelimit: RateLimit<ChecklistRatelimit>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    let items = state
        .db
        .checklists
        .get_by_owner(user_id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = Checklists::fill_out(state.db.clone(), items)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::OK, Json(filled_out)).into_response())
}
