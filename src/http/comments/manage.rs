use crate::db::resource::comment::comment::Comments;
use crate::db::resource::comment::comment_def::CommentFilledOut;
use crate::extractors::authenticator::AuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;
use crate::extractors::ratelimit::{CommentRatelimit, RateLimit};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCommentRequest {
    pub body: String,
    pub parent_comment_id: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCommentRequest {
    pub body: String,
}

#[utoipa::path(
    post,
    path = "/comment-threads/{id}/comments",
    tag = "Comments",
    summary = "Create a comment or reply",
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created successfully",
            body = CommentFilledOut),
        (status = 400, description = "Invalid comment"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_comment(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<CommentRatelimit>,
    Json(request): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentFilledOut>), ApiError> {
    if request.body.trim().is_empty() {
        return Err(ApiError::GenericServerError(
            "Comment body cannot be empty".to_string(),
        ));
    }

    // Verify that a parent comment belongs to the same thread.
    if let Some(parent_id) = request.parent_comment_id {
        let parent = state
            .db
            .comments
            .get(parent_id)
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?
            .ok_or_else(|| {
                ApiError::GenericServerError("Can't find parent comment id".to_string())
            })?;

        if parent.thread != id {
            return Err(ApiError::GenericServerError(
                "Parent comment belongs to a different thread".to_string(),
            ));
        }
    }

    let comment = state
        .db
        .comments
        .create(id, user_id, request.body.trim(), request.parent_comment_id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let filled_out = Comments::fill_out(state.db.clone(), comment, Some(user_id))
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok((StatusCode::CREATED, Json(filled_out)))
}

#[utoipa::path(
    patch,
    path = "/comments/{id}",
    tag = "Comments",
    summary = "Update a comment",
    params(
        ("id" = i64, Path, description = "Comment ID")
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 204, description = "Comment updated successfully"),
        (status = 403, description = "Not the comment author"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_comment(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<CommentRatelimit>,
    Json(request): Json<UpdateCommentRequest>,
) -> Result<StatusCode, ApiError> {
    if request.body.trim().is_empty() {
        return Err(ApiError::GenericServerError(
            "Comment body cannot be empty".to_string(),
        ));
    }

    let mut comment = state
        .db
        .comments
        .get(id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
        .ok_or_else(|| ApiError::GenericServerError("Can't find parent comment id".to_string()))?;

    if comment.author_id != user_id {
        return Err(ApiError::GenericServerError(
            "You cannot edit another user's comment".to_string(),
        ));
    }

    comment.body = request.body.trim().to_string();
    comment.edited = true;
    comment.edited_at = Some(Utc::now().to_rfc3339());

    state
        .db
        .comments
        .update(&comment)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/comments/{id}",
    tag = "Comments",
    summary = "Delete a comment",
    params(
        ("id" = i64, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment deleted successfully"),
        (status = 403, description = "Not the comment author"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_comment(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let comment = state
        .db
        .comments
        .get(id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
        .ok_or_else(|| ApiError::GenericServerError("Can't find parent comment id".to_string()))?;

    if comment.author_id != user_id {
        return Err(ApiError::GenericServerError(
            "You cannot delete another user's comment".to_string(),
        ));
    }

    state
        .db
        .comments
        .delete(id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
