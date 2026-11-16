use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;
use crate::fetchers::model_manufacturers::KNOWN_MANUFACTURERS;

#[derive(Debug, Serialize, ToSchema)]
pub struct MetaResponse {
    pub cdn: String,
    pub presets: Presets,
    pub model_bus_manufacturers: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Presets {
    #[serde(rename = "Bee Network")]
    pub bee_network: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/meta",
    tag = "Meta",
    summary = "Get API metadata",
    responses(
        (status = 200, description = "API metadata", body = MetaResponse)
    )
)]
pub async fn meta() -> Response {
    Json(MetaResponse {
        cdn: "https://transportia-cdn.dawn.rest".to_string(),
        presets: Presets {
            bee_network: vec![
                "BNSM".to_string(),
                "BNML".to_string(),
                "BNGN".to_string(),
                "BNVB".to_string(),
                "BNFB".to_string(),
                "BNDB".to_string(),
            ],
        },
        model_bus_manufacturers: KNOWN_MANUFACTURERS.iter().map(|x| x.to_string()).collect(),
    })
        .into_response()
}