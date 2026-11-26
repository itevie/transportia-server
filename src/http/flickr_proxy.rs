use axum::extract::{Query, State};
use axum::Json;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};
use crate::http::errors::ApiError;
use crate::http::HttpState;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct FlickrQuery {
    pub tags: String,
}

#[utoipa::path(
    get,
    path = "/flickr",
    tag = "Photos",
    summary = "Search Flickr photos",
    params(FlickrQuery),
    responses(
        (
            status = 200,
            description = "Flickr public photos feed",
            body = Value
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn flickr_photos(
    State(_state): State<HttpState>,
    Query(query): Query<FlickrQuery>,
) -> Result<Json<Value>, ApiError> {
    let response = reqwest::Client::new()
        .get("https://www.flickr.com/services/feeds/photos_public.gne")
        .query(&[
            ("tags", query.tags.as_str()),
            ("format", "json"),
            ("nojsoncallback", "1"),
        ])
        .send()
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::GenericServerError(format!(
            "Flickr returned HTTP {}",
            response.status()
        )));
    }

    let json = response
        .json::<Value>()
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(Json(json))
}