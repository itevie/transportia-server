use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, IntoParams, FromRow)]
pub struct TramAnnouncementAudio {
    pub id: i64,
    pub file_name: String,
    pub audio_group: String,
    pub label: Option<String>,
    pub operator_id: Option<i64>,
    pub route_id: Option<i64>,
    pub next_stop_id: Option<i64>,
    pub created_at: String,
}
