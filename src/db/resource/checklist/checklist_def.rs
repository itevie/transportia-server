
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[repr(i32)]
pub enum ChecklistType {
    Service = 0,
    Vehicle = 1,
}

impl sqlx::Type<sqlx::Postgres> for ChecklistType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i32 as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <i32 as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ChecklistType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i32 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&(*self as i32), buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ChecklistType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <i32 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;

        match value {
            0 => Ok(Self::Service),
            1 => Ok(Self::Vehicle),
            _ => Err(format!("Invalid ChecklistType value: {value}").into()),
        }
    }
}

impl FromStr for ChecklistType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "service" => Ok(Self::Service),
            "vehicle" => Ok(Self::Vehicle),
            _ => Err(format!("Invalid checklist type: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Checklist {
    pub id: i64,

    pub title: String,
    pub description: Option<String>,

    pub owner_id: i64,

    pub checklist_type: ChecklistType,

    pub auto_check_photos: bool,

    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct ChecklistFilledOut {
    pub id: i64,

    pub title: String,
    pub description: Option<String>,

    pub owner_id: i64,

    pub checklist_type: ChecklistType,

    pub auto_check_photos: bool,

    pub item_count: i64,
    pub completed_item_count: i64,

    pub created_at: String,
}
