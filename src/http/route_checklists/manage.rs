use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::ToSchema;
use crate::auth::quick::check_route_checklist;
use crate::db::resource::checklist::checklist_def::RouteChecklist;
use crate::extractors::authenticator::AuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RouteChecklistCreate {
    pub title: String,
    pub description: Option<String>,
}

#[utoipa::path(
    post,
    path = "/route-checklists",
    tag = "Route Checklists",
    summary = "Create a route checklist",
    description = "Creates a new route checklist for the authenticated user.",
    request_body = RouteChecklistCreate,
    responses(
        (
            status = 201,
            description = "Route checklist created successfully",
            body = RouteChecklist
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
pub async fn create_route_checklist(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Json(data): Json<RouteChecklistCreate>,
) -> Result<Response, ApiError> {
    let checklist = state
        .db
        .route_checklists
        .create(
            &data.title,
            data.description.as_deref(),
            user_id,
            &chrono::Utc::now().to_rfc3339(),
        )
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::CREATED, Json(checklist)).into_response())
}

#[utoipa::path(
    delete,
    path = "/route-checklists/{id}",
    tag = "Route Checklists",
    summary = "Delete a route checklist",
    description = "Deletes a route checklist owned by the authenticated user.",
    params(
        (
            "id" = i64,
            Path,
            description = "Route checklist ID"
        )
    ),
    responses(
        (
            status = 204,
            description = "Route checklist deleted successfully"
        ),
        (
            status = 401,
            description = "User does not own the checklist"
        ),
        (
            status = 404,
            description = "Route checklist not found"
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
pub async fn delete_route_checklist(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    check_route_checklist(state.db.clone(), id, user_id).await?;

    state
        .db
        .route_checklists
        .delete(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}