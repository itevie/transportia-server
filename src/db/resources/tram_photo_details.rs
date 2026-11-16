
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::tram::TramFilledOut;
use crate::db::resources::tram_livery::TramLivery;
use crate::db::resources::tram_type::TramType;
use crate::db::resources::tram_operator::TramOperator;

// TODO: Add head sign to this, e.g. Eccles
// TODO: And also add the tram route
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TramPhotoDetails {
    pub id: i64,
    pub matched_vehicle_id: Option<i64>,
    pub override_match: bool,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub vehicle_type_id: Option<i64>,
    pub operator_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TramPhotoDetailsUpdate {
    pub matched_vehicle_id: Option<i64>,
    pub override_match: Option<bool>,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub vehicle_type_id: Option<i64>,
    pub operator_id: Option<i64>,
}
