use crate::db::resource::photo::bus_photo_details_def::BusPhotoDetailsFilledOut;
use crate::db::resource::photo::model_bus_photo_details_def::ModelBusPhotoDetailsFilledOut;
use crate::db::resource::photo::tram_photo_details_def::TramPhotoDetailsFilledOut;
use crate::db::resource::user::def::User;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, ToSchema)]
#[repr(i64)]
pub enum PhotoType {
    Bus = 0,
    ModelBus = 1,
    Train = 2,
    Tram = 3,
}

impl sqlx::Type<sqlx::Postgres> for PhotoType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <i64 as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for PhotoType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&(*self as i64), buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for PhotoType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <i64 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;

        match value {
            0 => Ok(Self::Bus),
            1 => Ok(Self::ModelBus),
            2 => Ok(Self::Train),
            3 => Ok(Self::Tram),
            _ => Err(format!("Invalid PhotoType value: {value}").into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Photo {
    pub id: i64,
    pub author_id: i64,
    pub photo_type: PhotoType,
    pub obj_details_id: Option<i64>,
    pub comment_thread: Option<i64>,
    pub file_name: String,
    pub uploaded_id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhotoFilledOut {
    pub id: i64,
    pub author_id: i64,
    pub author: User,
    pub photo_type: PhotoType,
    pub obj_details_id: Option<i64>,
    pub obj_details: Option<PhotoDetails>,
    pub comment_thread: Option<i64>,
    pub file_name: String,
    pub uploaded_id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,

    pub is_favourited: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhotoUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum PhotoDetails {
    Bus(BusPhotoDetailsFilledOut),
    Tram(TramPhotoDetailsFilledOut),
    ModelBus(ModelBusPhotoDetailsFilledOut),
}

// #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
// pub struct FilledOutPhotoBus {
//     pub id: i64,
//     pub author_id: i64,
//     pub author: User,
//     pub photo_type: PhotoType,
//     pub obj_details_id: Option<i64>,
//     pub obj_details: Option<BusPhotoDetailsFilledOut>,
//     pub comment_thread: Option<i64>,
//     pub file_name: String,
//     pub uploaded_id: String,
//     pub title: String,
//     pub description: Option<String>,
//     pub created_at: String,
//
//     pub is_favourited: Option<bool>
// }
