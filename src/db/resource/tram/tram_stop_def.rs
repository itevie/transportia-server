use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TramStop {
    pub id: i64,
    pub name: String,
    pub operator_id: Option<i64>,
    pub created_at: String,
}
