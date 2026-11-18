use crate::db::resource::comment::comment::Comments;
use crate::db::resource::comment::comment_def::{Comment, CommentFilledOut};
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(
    get,
    path = "/comment-threads/{id}",
    tag = "Comments",
    summary = "Get comments for a comment thread",
    responses(
        (status = 200, description = "Comments retrieved successfully",
            body = Vec<Comment>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_comments(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Result<Json<Vec<CommentFilledOut>>, ApiError> {
    let comments = state
        .db
        .comments
        .get_by_thread(id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let comments_filled_out =
        Comments::fill_out_many(state.db.clone(), comments, user.map(|x| x.user_id))
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(comments_filled_out))
}
