use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::db::resource::favourite::favourite_def::FavouriteType;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct FavouritesGroup {
    pub id: i64,
    pub favourite_type: FavouriteType,
    pub owner_id: i64,
    pub name: String,
    pub created_at: String,
}
