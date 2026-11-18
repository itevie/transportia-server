use crate::auth::quick::check_route_checklist;
use crate::db::resource::checklist::checklist_def::ChecklistType;
use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use crate::validation::checklists::check_checklist_obj;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(
    post,
    path = "/route-checklists/{checklist_id}/items/{line_id}",
    tag = "Route Checklists",
    summary = "Add a line to a route checklist",
    description = "Adds a specific service line to the authenticated user's route checklist.",
    params(
        (
            "checklist_id" = i64,
            Path,
            description = "Route checklist ID"
        ),
        (
            "obj_id" = String,
            Path,
            description = "Object ID, service or vehicle, based on checklist's type"
        )
    ),
    responses(
        (
            status = 204,
            description = "Line added to checklist successfully"
        ),
        (
            status = 401,
            description = "User does not own the checklist"
        ),
        (
            status = 404,
            description = "Checklist or line not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn create_route_checklist_item(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path((checklist_id, obj_id)): Path<(i64, String)>,
) -> Result<Response, ApiError> {
    let checklist = check_route_checklist(state.db.clone(), checklist_id, user_id).await?;
    check_checklist_obj(state.db.clone(), checklist, obj_id.clone()).await?;

    let added_at = chrono::Utc::now().to_rfc3339();

    let _ = state
        .db
        .checklist_items
        .create(checklist_id, &obj_id, &added_at)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    post,
    path = "/route-checklists/items/{checklist_id}/operators/{operator_id}",
    tag = "Route Checklists",
    summary = "Add an operator's lines to a checklist",
    description = "Adds all service lines belonging to an operator to the authenticated user's route checklist.",
    params(
        (
            "checklist_id" = i64,
            Path,
            description = "Route checklist ID"
        ),
        (
            "operator_id" = String,
            Path,
            description = "Operator's national code"
        )
    ),
    responses(
        (
            status = 204,
            description = "Operator lines added to checklist successfully"
        ),
        (
            status = 401,
            description = "User does not own the checklist"
        ),
        (
            status = 404,
            description = "Checklist not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn create_route_checklist_item_from_operators(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path((checklist_id, operator_id)): Path<(i64, String)>,
) -> Result<Response, ApiError> {
    let checklist = check_route_checklist(state.db.clone(), checklist_id, user_id).await?;

    let ids = match checklist.checklist_type {
        ChecklistType::Service => state
            .db
            .tx_lines
            .get_by_national_operator_code(operator_id.as_str())
            .await
            .map_err(|_e| {
                ApiError::GenericServerError("Failed to fetch lines by operator".to_string())
            })?
            .iter()
            .map(|x| x.tx_id.clone())
            .collect::<Vec<_>>(),
        ChecklistType::Vehicle => state
            .db
            .vehicles
            .get_by_national_operator_code(operator_id.as_str())
            .await
            .map_err(|_e| {
                ApiError::GenericServerError("Failed to fetch vehicles for operator".to_string())
            })?
            .iter()
            .map(|x| x.id.to_string())
            .collect::<Vec<_>>(),
    };

    let added_at = chrono::Utc::now().to_rfc3339();

    for id in ids {
        state
            .db
            .checklist_items
            .create(checklist_id, &id, &added_at)
            .await
            .map_err(|err| ApiError::Database(err.to_string()))?;
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    post,
    path = "/route-checklists/items/{checklist_id}/operators-bulk/",
    tag = "Route Checklists",
    summary = "Add multiple operators' lines to a checklist",
    description = "Adds all service lines belonging to multiple operators to the authenticated user's route checklist.",
    params(
        (
            "checklist_id" = i64,
            Path,
            description = "Route checklist ID"
        )
    ),
    request_body(
        content = Vec<String>,
        description = "List of operators' national codes",
        example = json!(["BNSM", "BNDB"])
    ),
    responses(
        (
            status = 204,
            description = "Operator lines added to checklist successfully"
        ),
        (
            status = 400,
            description = "Invalid request body"
        ),
        (
            status = 401,
            description = "User does not own the checklist"
        ),
        (
            status = 404,
            description = "Checklist not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn create_route_checklist_items_from_operators_bulk(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(checklist_id): Path<i64>,
    Json(operator_ids): Json<Vec<String>>,
) -> Result<Response, ApiError> {
    let checklist = check_route_checklist(state.db.clone(), checklist_id, user_id).await?;
    let added_at = chrono::Utc::now().to_rfc3339();

    for operator_id in operator_ids {
        let ids = match checklist.checklist_type {
            ChecklistType::Service => state
                .db
                .tx_lines
                .get_by_national_operator_code(operator_id.as_str())
                .await
                .map_err(|_e| {
                    ApiError::GenericServerError("Failed to fetch lines by operator".to_string())
                })?
                .iter()
                .map(|x| x.tx_id.clone())
                .collect::<Vec<_>>(),
            ChecklistType::Vehicle => state
                .db
                .vehicles
                .get_by_national_operator_code(operator_id.as_str())
                .await
                .map_err(|_e| {
                    ApiError::GenericServerError(
                        "Failed to fetch vehicles for operator".to_string(),
                    )
                })?
                .iter()
                .map(|x| x.id.to_string())
                .collect::<Vec<_>>(),
        };

        for id in ids {
            state
                .db
                .checklist_items
                .create(checklist_id, &id, &added_at)
                .await
                .map_err(|err| ApiError::Database(err.to_string()))?;
        }
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema)]
pub struct OperatorBulkCountOptions {
    pub operators: Vec<String>,
    pub checklist_type: ChecklistType,
}

#[utoipa::path(
    post,
    path = "/route-checklists/items/operators-bulk/count/",
    tag = "Route Checklists",
    summary = "Count multiple operators' lines",
    description = "Returns the total number of service lines belonging to the specified operators without adding them to a checklist.",
    request_body(
        content = Vec<String>,
        description = "List of operators' national codes",
        example = json!(["BNSM", "BNDB"])
    ),
    responses(
        (
            status = 200,
            description = "Total number of lines",
            body = usize,
            example = json!(42)
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn count_lines_from_operators_bulk(
    State(state): State<HttpState>,
    Json(json): Json<OperatorBulkCountOptions>,
) -> Result<Response, ApiError> {
    let mut count = 0usize;

    for operator_id in json.operators {
        let ids = match json.checklist_type {
            ChecklistType::Service => state
                .db
                .tx_lines
                .get_by_national_operator_code(operator_id.as_str())
                .await
                .map_err(|_e| {
                    ApiError::GenericServerError("Failed to fetch lines by operator".to_string())
                })?
                .iter()
                .map(|x| x.tx_id.clone())
                .collect::<Vec<_>>(),
            ChecklistType::Vehicle => state
                .db
                .vehicles
                .get_by_national_operator_code(operator_id.as_str())
                .await
                .map_err(|_e| {
                    ApiError::GenericServerError(
                        "Failed to fetch vehicles for operator".to_string(),
                    )
                })?
                .iter()
                .map(|x| x.id.to_string())
                .collect::<Vec<_>>(),
        };

        count += ids.len();
    }

    Ok(axum::Json(count).into_response())
}

#[utoipa::path(
    patch,
    path = "/route-checklists/{checklist_id}/items/{item_id}/toggle-completed",
    tag = "Route Checklists",
    summary = "Toggle checklist item completion",
    description = "Marks a route checklist item as completed, or un-completes it if it is already completed.",
    params(
        (
            "checklist_id" = i64,
            Path,
            description = "Route checklist ID"
        ),
        (
            "item_id" = i64,
            Path,
            description = "Route checklist item ID"
        )
    ),
    responses(
        (
            status = 204,
            description = "Checklist item completion updated successfully"
        ),
        (
            status = 401,
            description = "User does not own the checklist"
        ),
        (
            status = 404,
            description = "Checklist or item not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn complete_route_checklist_item(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path((checklist_id, item_id)): Path<(i64, i64)>,
) -> Result<Response, ApiError> {
    check_route_checklist(state.db.clone(), checklist_id, user_id).await?;

    let item = state
        .db
        .checklist_items
        .get(item_id)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(item_id.to_string()))?;

    let completed_at = chrono::Utc::now().to_rfc3339();

    state
        .db
        .checklist_items
        .set_completed(
            item.id,
            if let Some(_) = item.completed_at {
                None
            } else {
                Some(&completed_at)
            },
        )
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
