use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::route_checklist_item::RouteChecklistItemFilledOut;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RouteChecklist {
    pub id: i64,

    pub title: String,
    pub description: Option<String>,

    pub owner_id: i64,

    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RouteChecklistFilledOut {
    pub id: i64,

    pub title: String,
    pub description: Option<String>,

    pub owner_id: i64,

    pub item_count: i64,
    pub completed_item_count: i64,

    pub created_at: String,
}