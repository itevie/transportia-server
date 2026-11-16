use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// These are provided from https://bustimes.org/api/vehicletypes
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BtVehicleType {
    pub id: i64,
    pub name: String,
    pub style: String,
    pub fuel: String,
    pub double_decker: bool,
    pub coach: bool,
    pub electric: bool
}