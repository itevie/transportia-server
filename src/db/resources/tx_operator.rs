use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TxOperator {
    pub id: String,
    pub national_code: Option<String>,
    pub code: Option<String>,
    pub short_name: String,
    pub licence_number: Option<String>,
    pub comment_thread: Option<i64>
}