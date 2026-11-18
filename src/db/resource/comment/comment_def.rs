use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::db::resource::user::def::User;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CommentThread {
    pub id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Comment {
    pub id: i64,
    pub thread: i64,
    pub author_id: i64,
    pub body: String,
    pub edited: bool,
    pub edited_at: Option<String>,
    pub parent_comment_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CommentReaction {
    pub id: i64,
    pub user_id: i64,
    pub comment_id: i64,
    pub reaction_type: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CommentFilledOut {
    pub id: i64,
    pub thread: i64,
    pub author_id: i64,
    pub author: User,
    pub body: String,
    pub edited: bool,
    pub edited_at: Option<String>,
    pub parent_comment_id: Option<i64>,
    pub created_at: String,
    pub is_favourited: Option<bool>
}