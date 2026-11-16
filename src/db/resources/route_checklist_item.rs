use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::tx_line::TxLine;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RouteChecklistItem {
    pub id: i64,

    pub checklist_id: i64,
    pub line_id: String,

    pub added_at: String,
    pub completed_at: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RouteChecklistItemFilledOut {
    pub id: i64,

    pub checklist_id: i64,
    pub line_id: String,
    pub line: TxLine,

    pub added_at: String,
    pub completed_at: Option<String>,
}