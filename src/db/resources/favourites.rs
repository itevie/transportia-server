use std::str::FromStr;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::resources::bt_livery::BtLivery;
use crate::db::resources::bt_vehicle::{BtVehicle, FilledOutBtVehicle};
use crate::db::resources::bt_vehicle_type::BtVehicleType;
use crate::db::resources::comments::CommentFilledOut;
use crate::db::resources::model_bus::ModelBusFilledOut;
use crate::db::resources::photo::{PhotoFilledOut};
use crate::db::resources::tram::TramFilledOut;
use crate::db::resources::tx_line::TxLine;
use crate::db::resources::tx_operator::TxOperator;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[repr(i64)]
pub enum FavouriteType {
    Photo = 0,
    Operator = 1,
    Vehicle = 2,
    VehicleType = 3,
    Livery = 4,
    Service = 5,
    Comment = 6,
    Tram = 7,
    ModelBus = 8
}

impl FromStr for FavouriteType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "photo" => Ok(Self::Photo),
            "operator" => Ok(Self::Operator),
            "vehicle" => Ok(Self::Vehicle),
            "vehicle_type" => Ok(Self::VehicleType),
            "livery" => Ok(Self::Livery),
            "service" => Ok(Self::Service),
            "comment" => Ok(Self::Comment),
            "tram" => Ok(Self::Tram),
            "model_bus" => Ok(Self::ModelBus),
            _ => Err(format!("Invalid favourite type: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Favourite {
    pub id: i64,
    pub user_id: i64,
    pub target_id: String,
    pub target_type: FavouriteType,
    pub created_at: String,
    pub group: Option<i64>
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FavouriteFilled {
    pub id: i64,
    pub user_id: i64,
    pub target_id: String,
    pub target_type: FavouriteType,
    pub created_at: String,
    pub group: Option<i64>,
    pub object: FavouriteObject
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum FavouriteObject {
    Vehicle(FilledOutBtVehicle),
    VehicleType(BtVehicleType),
    Livery(BtLivery),
    Service(TxLine),
    Operator(TxOperator),
    BusPhoto(PhotoFilledOut),
    Comment(CommentFilledOut),
    Tram(TramFilledOut),
    ModelBus(ModelBusFilledOut)
}