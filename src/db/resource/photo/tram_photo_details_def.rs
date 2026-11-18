use crate::db::resource::tram::tram_def::TramFilledOut;
use crate::db::resource::tram::tram_livery_def::TramLivery;
use crate::db::resource::tram::tram_operator_def::TramOperator;
use crate::db::resource::tram::tram_type_def::TramType;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// TODO: Add head sign to this, e.g. Eccles
// TODO: And also add the tram route
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TramPhotoDetails {
    pub id: i64,
    pub matched_vehicle_id: Option<i64>,
    pub override_match: bool,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub vehicle_type_id: Option<i64>,
    pub operator_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TramPhotoDetailsFilledOut {
    pub id: i64,
    pub matched_vehicle_id: Option<i64>,
    pub matched_vehicle: Option<TramFilledOut>,
    pub override_match: bool,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub livery: Option<TramLivery>,
    pub vehicle_type_id: Option<i64>,
    pub vehicle_type: Option<TramType>,
    pub operator_id: Option<i64>,
    pub operator: Option<TramOperator>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TramPhotoDetailsUpdate {
    pub matched_vehicle_id: Option<i64>,
    pub override_match: Option<bool>,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub vehicle_type_id: Option<i64>,
    pub operator_id: Option<i64>,
}
