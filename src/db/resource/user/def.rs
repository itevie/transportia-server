use crate::db::util::serde::double_option;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use bitflags::bitflags;
use sqlx::FromRow;
use crate::define_update;
use crate::validation::fields::FieldValidationType;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct UserFlags: u32 {
        const ADMIN = 1 << 0;
        const CAN_EDIT_OBJ = 1 << 1;
        const CAN_CREATE_OBJ = 1 << 2;
        const CAN_DELETE_OBJ = 1 << 3;
        const CAN_EDIT_USER_PHOTO_DETAILS = 1 << 4;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct User {
    pub id: i64,

    pub email: String,
    pub username: String,

    pub display_name: Option<String>,
    pub bio: Option<String>,

    pub password_hash: String,
    
    pub public_favourites: bool,

    pub trusted_user: bool,
    pub admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub public_favourites: Option<bool>,
    
    // Nullable
    #[serde(default, deserialize_with = "double_option")]
    pub display_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub bio: Option<Option<String>>,
}

