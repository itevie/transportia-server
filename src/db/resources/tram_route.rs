use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TramRoute {
    pub id: i64,
    pub name: String,
    pub start_stop_id: i64,
    pub end_stop_id: i64,
    pub operator_id: i64,
    pub created_at: String,
}