use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utoipa::ToSchema;
use crate::db::resources::bus_photo_details::BusPhotoDetailsFilledOut;
use crate::db::resources::model_bus_photo_details::ModelBusPhotoDetailsFilledOut;
use crate::db::resources::tram_photo_details::TramPhotoDetailsFilledOut;
use crate::db::resources::user::User;

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, ToSchema)]
#[repr(i64)]
pub enum PhotoType {
    Bus = 0,
    ModelBus = 1,
    Train = 2,
    Tram = 3
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

    pub is_favourited: Option<bool>
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