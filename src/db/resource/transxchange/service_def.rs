use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams, FromRow)]
pub struct TxService {
    pub id: String,
    pub service_code: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,

    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,

    pub bank_holidays_non_operation: String,
    pub special_days_of_operation: String,
    pub special_days_of_non_operation: String,
    pub serviced_organisation_refs: String,

    pub operator: Option<String>,
}
