use crate::db::resource::bustimes::livery_def::BtLivery;
use crate::db::resource::bustimes::vehicle_def::FilledOutBtVehicle;
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use crate::db::resource::comment::comment_def::CommentFilledOut;
use crate::db::resource::model_bus::model_bus_def::ModelBusFilledOut;
use crate::db::resource::photo::photo_def::PhotoFilledOut;
use crate::db::resource::tram::tram_def::TramFilledOut;
use crate::db::resource::transxchange::line_def::TxLine;
use crate::db::resource::transxchange::operator_def::TxOperator;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use utoipa::ToSchema;

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
    ModelBus = 8,
}

impl sqlx::Type<sqlx::Postgres> for FavouriteType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <i64 as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for FavouriteType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&(*self as i64), buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for FavouriteType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <i64 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;

        match value {
            0 => Ok(Self::Photo),
            1 => Ok(Self::Operator),
            2 => Ok(Self::Vehicle),
            3 => Ok(Self::VehicleType),
            4 => Ok(Self::Livery),
            5 => Ok(Self::Service),
            6 => Ok(Self::Comment),
            7 => Ok(Self::Tram),
            8 => Ok(Self::ModelBus),
            _ => Err(format!("Invalid FavouriteType value: {value}").into()),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Favourite {
    pub id: i64,
    pub user_id: i64,
    pub target_id: String,
    pub target_type: FavouriteType,
    pub created_at: String,
    pub group: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct FavouriteFilled {
    pub id: i64,
    pub user_id: i64,
    pub target_id: String,
    pub target_type: FavouriteType,
    pub created_at: String,
    pub group: Option<i64>,
    pub object: FavouriteObject,
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
    ModelBus(ModelBusFilledOut),
}
