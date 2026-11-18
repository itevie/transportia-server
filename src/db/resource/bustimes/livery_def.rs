use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// These are provided from https://bustimes.org/api/liveries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct BtLivery {
    pub id: i64,
    pub name: String,
    pub left_css: String,
    pub right_css: String,
    pub white_text: bool,
    pub text_colour: String,
    pub stroke_colour: String,
}