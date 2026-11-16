use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::resources::model_bus::ModelBusFilledOut;
use crate::db::resources::model_bus_ownership::{ModelBusOwnership, ModelBusOwnershipFilledOut};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelBusPhotoDetails {
    pub id: i64,
    pub matched_model_id: Option<i64>,
    pub ownership: Option<i64>,
    pub override_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelBusPhotoDetailsFilledOut {
    pub id: i64,
    pub matched_model_id: Option<i64>,
    pub matched_model: Option<ModelBusFilledOut>,
    pub ownership: Option<i64>,
    pub ownership_details: Option<ModelBusOwnershipFilledOut>,
    pub override_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelBusPhotoDetailsUpdate {
    pub matched_model_id: Option<i64>,
    pub ownership: Option<i64>,
    pub override_match: Option<bool>,
}