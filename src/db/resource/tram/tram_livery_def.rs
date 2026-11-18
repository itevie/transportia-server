use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TramLivery {
    pub id: i64,
    pub name: String,
    pub css: Option<String>,
    pub text_colour: Option<String>,
}