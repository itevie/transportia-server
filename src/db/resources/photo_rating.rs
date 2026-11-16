use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhotoRating {
    pub id: i64,

    pub photo_id: i64,
    pub user_id: i64,

    /// 1-5
    pub rating: i64,

    pub created_at: String,
}