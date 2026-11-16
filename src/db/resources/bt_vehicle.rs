use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::bt_livery::BtLivery;
use crate::db::resources::bt_vehicle_type::BtVehicleType;
use crate::db::resources::photo::Photo;
use crate::db::resources::tx_operator::TxOperator;

// These are provided from https://bustimes.org/api/vehicles
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BtVehicle {
    pub id: i64,
    pub slug: String,
    pub fleet: String,
    pub reg: String,
    pub previous_reg: String,
    pub vehicle_type_id: Option<i64>,
    pub livery_id: Option<i64>,
    pub branding: String,
    pub operator_id: String,
    pub garage_id: Option<String>,
    pub name: String,
    pub notes: String,
    pub withdrawn: bool,
    pub training_vehicle: bool,
    pub demonstrator: bool,
    pub open_top: bool,
    pub comment_thread: Option<i64>
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilledOutBtVehicle {
    pub id: i64,
    pub slug: String,
    pub fleet: String,
    pub reg: String,
    pub previous_reg: String,
    pub vehicle_type_id: Option<i64>,
    pub vehicle_type: Option<BtVehicleType>,
    pub livery_id: Option<i64>,
    pub livery: Option<BtLivery>,
    pub branding: String,
    pub operator_id: String,
    pub operator: Option<TxOperator>,
    pub garage_id: Option<String>,
    pub name: String,
    pub notes: String,
    pub withdrawn: bool,
    pub training_vehicle: bool,
    pub demonstrator: bool,
    pub open_top: bool,
    pub photo: Option<Photo>,
    pub comment_thread: Option<i64>,

    pub is_favourited: Option<bool>,
}