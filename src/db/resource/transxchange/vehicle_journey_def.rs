use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams, FromRow)]
pub struct TxVehicleJourney {
    pub id: String,
    pub operator_ref: Option<String>,
    pub service_ref: Option<String>,
    pub line_ref: Option<String>,
    pub journey_pattern_ref: Option<String>,
    pub departure_time: Option<String>,
    pub garage_ref: Option<String>,

    pub block_description: Option<String>,
    pub block_number: Option<String>,

    pub ticket_machine_service_code: Option<String>,
    pub ticket_machine_journey_code: Option<String>,

    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
}
