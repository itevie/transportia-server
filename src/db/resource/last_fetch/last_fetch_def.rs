use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LastFetches {
    pub bt_liveries: Option<String>,
    pub bt_operators: Option<String>,
    pub bt_vehicle_types: Option<String>,
    pub bt_vehicles: Option<String>,
    pub models_mbc: Option<String>,
    pub models_ttcdiecast: Option<String>,
}
