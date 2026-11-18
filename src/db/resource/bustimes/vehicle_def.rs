use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::db::resource::bustimes::livery_def::BtLivery;
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use crate::db::resource::photo::photo_def::Photo;
use crate::db::resource::transxchange::operator_def::TxOperator;

// These are provided from https://bustimes.org/api/vehicles
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
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