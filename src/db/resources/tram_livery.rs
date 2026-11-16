use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TramLivery {
    pub id: i64,
    pub name: String,
    pub css: Option<String>,
    pub text_colour: Option<String>,
}