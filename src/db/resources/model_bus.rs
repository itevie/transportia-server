
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::bt_livery::BtLivery;
use crate::db::resources::bt_vehicle::BtVehicle;
use crate::db::resources::bt_vehicle_type::BtVehicleType;
use crate::db::resources::photo::Photo;
use crate::db::resources::tx_line::TxLine;
use crate::db::resources::tx_operator::TxOperator;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelBus {
    pub id: i64,

    pub product_code: String,
    pub manufacturer: String,
    pub scale: i64,

    pub operator: Option<String>,
    pub operator_link: Option<String>,

    pub livery: Option<String>,
    pub livery_link: Option<i64>,

    pub fleet_number: Option<String>,
    pub registration: Option<String>,
    pub vehicle_link: Option<i64>,

    pub route_number: Option<String>,
    pub destination: Option<String>,
    pub route_link: Option<String>,

    pub vehicle_type: Option<String>,
    pub vehicle_type_link: Option<i64>,

    pub has_certificate: bool,
    pub certificate_max: i64,

    pub released: Option<String>,

    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelBusFilledOut {
    pub id: i64,

    pub product_code: String,
    pub manufacturer: String,
    pub scale: i64,

    pub operator: Option<String>,
    pub operator_link: Option<String>,
    pub operator_obj: Option<TxOperator>,

    pub livery: Option<String>,
    pub livery_link: Option<i64>,
    pub livery_obj: Option<BtLivery>,

    pub fleet_number: Option<String>,
    pub registration: Option<String>,
    pub vehicle_link: Option<i64>,
    pub vehicle_obj: Option<BtVehicle>,

    pub route_number: Option<String>,
    pub destination: Option<String>,
    pub route_link: Option<String>,
    pub route_obj: Option<TxLine>,

    pub vehicle_type: Option<String>,
    pub vehicle_type_link: Option<i64>,
    pub vehicle_type_obj: Option<BtVehicleType>,

    pub has_certificate: bool,
    pub certificate_max: i64,

    pub released: Option<String>,

    pub photo: Option<Photo>,

    pub created_at: String,
    pub updated_at: String,
}