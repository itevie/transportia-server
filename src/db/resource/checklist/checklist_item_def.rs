use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::db::resource::transxchange::line_def::TxLine;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct RouteChecklistItem {
    pub id: i64,

    pub checklist_id: i64,
    pub line_id: String,

    pub added_at: String,
    pub completed_at: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct RouteChecklistItemFilledOut {
    pub id: i64,

    pub checklist_id: i64,
    pub line_id: String,
    pub line: TxLine,

    pub added_at: String,
    pub completed_at: Option<String>,
}