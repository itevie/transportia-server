use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::db::resources::photo::Photo;
use crate::db::resources::tram_livery::TramLivery;
use crate::db::resources::tram_operator::TramOperator;
use crate::db::resources::tram_type::TramType;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Tram {
    pub id: i64,
    pub operator_id: Option<i64>,
    pub fleet: String,
    pub livery_id: Option<i64>,
    pub tram_type_id: Option<i64>,
    pub name: Option<String>,
    pub garage: Option<String>,
    pub withdrawn: bool,
    pub branding: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
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
    pub is_favourited: Option<bool>
}