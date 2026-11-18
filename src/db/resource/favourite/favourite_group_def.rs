use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct FavouritesGroup {
    pub id: i64,
    pub favourite_type: i64,
    pub owner_id: i64,
    pub name: String,
    pub created_at: String,
}
