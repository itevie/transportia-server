use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::bt_livery::BtLivery;
use crate::db::resources::bt_vehicle::FilledOutBtVehicle;
use crate::db::resources::bt_vehicle_type::BtVehicleType;
use crate::db::resources::tx_line::TxLine;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BusPhotoDetailsFilledOut {
    pub id: i64,
    pub matched_vehicle_id: Option<i64>,
    pub matched_vehicle: Option<FilledOutBtVehicle>,
    pub override_match: bool,
    pub reg: Option<String>,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub livery: Option<BtLivery>,
    pub vehicle_type_id: Option<i64>,
    pub vehicle_type: Option<BtVehicleType>,
    pub route_id: Option<String>,
    pub route: Option<TxLine>
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BusPhotoDetailsUpdate {
    pub matched_vehicle_id: Option<i64>,
    pub override_match: Option<bool>,
    pub reg: Option<String>,
    pub fleet: Option<String>,
    pub livery_id: Option<i64>,
    pub vehicle_type_id: Option<i64>,
    pub route_id: Option<String>,
}