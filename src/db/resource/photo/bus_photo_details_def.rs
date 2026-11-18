use crate::db::resource::bustimes::livery_def::BtLivery;
use crate::db::resource::bustimes::vehicle_def::BtVehicleFilleOut;
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use crate::db::resource::transxchange::line_def::TxLine;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct BusPhotoDetails {
    pub id: i64,
    pub matched_vehicle_id: Option<i64>,
    pub override_match: bool,
    pub reg: Option<String>,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub vehicle_type_id: Option<i64>,
    pub route_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct BusPhotoDetailsFilledOut {
    pub id: i64,
    pub matched_vehicle_id: Option<i64>,
    pub matched_vehicle: Option<BtVehicleFilleOut>,
    pub override_match: bool,
    pub reg: Option<String>,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub livery: Option<BtLivery>,
    pub vehicle_type_id: Option<i64>,
    pub vehicle_type: Option<BtVehicleType>,
    pub route_id: Option<String>,
    pub route: Option<TxLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct BusPhotoDetailsUpdate {
    pub matched_vehicle_id: Option<i64>,
    pub override_match: Option<bool>,
    pub reg: Option<String>,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub vehicle_type_id: Option<i64>,
    pub route_id: Option<String>,
}
