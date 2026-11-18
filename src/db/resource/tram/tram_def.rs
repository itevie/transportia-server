use crate::db::resource::photo::photo_def::Photo;
use crate::db::resource::tram::tram_livery_def::TramLivery;
use crate::db::resource::tram::tram_operator_def::TramOperator;
use crate::db::resource::tram::tram_type_def::TramType;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Tram {
    pub id: i64,
    pub operator_id: Option<i64>,
    pub fleet: String,
    pub livery_id: Option<i64>,
    pub tram_type_id: Option<i64>,
    pub name: Option<String>,
    pub garage: Option<String>,
    pub withdrawn: bool,
    pub branding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams, FromRow)]
pub struct TramFilledOut {
    pub id: i64,
    pub operator_id: Option<i64>,
    pub operator: Option<TramOperator>,
    pub fleet: String,
    pub livery_id: Option<i64>,
    pub livery: Option<TramLivery>,
    pub tram_type_id: Option<i64>,
    pub tram_type: Option<TramType>,
    pub name: Option<String>,
    pub garage: Option<String>,
    pub withdrawn: bool,
    pub branding: Option<String>,
    pub photo: Option<Photo>,
    pub is_favourited: Option<bool>,
}
