use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TramOperator {
    pub id: i64,
    pub name: String,
    pub location: Option<String>,
    pub website: Option<String>,
}