use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::db::resource::model_bus::model_bus_def::ModelBusFilledOut;
use crate::db::resource::model_bus::model_bus_ownership_def::ModelBusOwnershipFilledOut;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct ModelBusPhotoDetails {
    pub id: i64,
    pub matched_model_id: Option<i64>,
    pub ownership: Option<i64>,
    pub override_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct ModelBusPhotoDetailsFilledOut {
    pub id: i64,
    pub matched_model_id: Option<i64>,
    pub matched_model: Option<ModelBusFilledOut>,
    pub ownership: Option<i64>,
    pub ownership_details: Option<ModelBusOwnershipFilledOut>,
    pub override_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct ModelBusPhotoDetailsUpdate {
    pub matched_model_id: Option<i64>,
    pub ownership: Option<i64>,
    pub override_match: Option<bool>,
}