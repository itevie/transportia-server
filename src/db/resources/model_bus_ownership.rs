use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::model_bus::ModelBusFilledOut;
use crate::db::resources::photo::Photo;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelBusOwnership {
    pub id: i64,
    pub user_id: i64,
    pub model_bus_id: i64,
    pub certificate_number: Option<i64>,
    pub bought_at: Option<String>,
    pub defects: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelBusOwnershipFilledOut {
    pub id: i64,
    pub user_id: i64,
    pub model_bus_id: i64,
    pub model_bus: ModelBusFilledOut,
    pub certificate_number: Option<i64>,
    pub bought_at: Option<String>,
    pub defects: Option<String>,
    pub photo: Option<Photo>,
    pub created_at: String,
}