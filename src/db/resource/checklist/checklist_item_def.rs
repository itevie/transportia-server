use crate::db::resource::bustimes::vehicle_def::BtVehicleFilleOut;
use crate::db::resource::transxchange::line_def::TxLine;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct ChecklistItem {
    pub id: i64,

    pub checklist_id: i64,
    pub obj_id: String,

    pub added_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChecklistItemFilledOut {
    pub id: i64,

    pub checklist_id: i64,
    pub obj_id: String,
    pub obj: ChecklistObj,

    pub added_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ChecklistObj {
    Service(TxLine),
    Vehicle(BtVehicleFilleOut),
}
