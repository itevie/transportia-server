use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct TxLine {
    pub tx_id: String,
    pub name: String,
    pub operator_code: Option<String>,
    pub operator_national_code: Option<String>,
    pub line_id: String,
    pub service_code: String,
    pub outbound_description: Option<String>,
    pub inbound_description: Option<String>,
}