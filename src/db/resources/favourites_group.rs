use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FavouritesGroup {
    pub id: i64,
    pub favourite_type: i64,
    pub owner_id: i64,
    pub name: String,
    pub created_at: String,
}