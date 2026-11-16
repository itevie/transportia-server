use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TramStop {
    pub id: i64,
    pub name: String,
    pub operator_id: Option<i64>,
    pub created_at: String,
}