use std::path::PathBuf;

use crate::http::HttpState;
use crate::http::errors::ApiError;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::Response;
use exif::{In, Tag};
use image::imageops::FilterType;
use image::metadata::Orientation;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PhotoImageQuery {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

fn image_response(bytes: Vec<u8>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(bytes.into())
        .unwrap()
}

#[utoipa::path(
    get,
    path = "/photos/{id}/image",
    summary = "Resize / fetch an image",
    tag = "Photos",
    params(
        (
            "id" = i64,
            Path,
            description = "Photo ID"
        ),
        (
            "width" = Option<u32>,
            Query,
            description = "Maximum image width. Defaults to 800."
        ),
        (
            "height" = Option<u32>,
            Query,
            description = "Maximum image height. If omitted, the aspect ratio is preserved."
        )
    ),
    responses(
        (
            status = 200,
            description = "Resized photo image",
            content_type = "image/jpeg"
        ),
        (
            status = 400,
            description = "Width and height must be greater than 0"
        ),
        (
            status = 404,
            description = "Photo not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_photo_image(
    State(state): State<HttpState>,
    Query(query): Query<PhotoImageQuery>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let width = query.width;
    let height = query.height;

    if width == Some(0) || height == Some(0) {
        return Err(ApiError::BadRequest(
            "width and height must be greater than 0".to_string(),
        ));
    }

    let photo = state
        .db
        .photos
        .get(id)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Photo not found".to_string()))?;

    // Requests without dimensions return the original image.
    if width.is_none() && height.is_none() {
        let url = format!("https://transportia-cdn.dawn.rest/{}", photo.uploaded_id);

        let response = reqwest::get(&url)
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::GenericServerError(format!(
                "Failed to fetch image: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        return Ok(image_response(bytes.to_vec()));
    }

    // Build the cache path and check it before accessing the CDN.
    let cache_name = match (width, height) {
        (Some(w), Some(h)) => format!("{id}_{w}x{h}.jpg"),
        (Some(w), None) => format!("{id}_w{w}.jpg"),
        (None, Some(h)) => format!("{id}_h{h}.jpg"),
        (None, None) => unreachable!(),
    };

    let cache_path = PathBuf::from("data/photo_cache").join(cache_name);

    match tokio::fs::read(&cache_path).await {
        Ok(bytes) => return Ok(image_response(bytes)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(ApiError::GenericServerError(err.to_string()));
        }
    }

    // Cache miss: fetch the original image.
    let url = format!("https://transportia-cdn.dawn.rest/{}", photo.uploaded_id);

    let response = reqwest::get(&url)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::GenericServerError(format!(
            "Failed to fetch image: HTTP {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    let mut image = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
        .decode()
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    // Preserve EXIF orientation.
    let orientation = {
        let mut cursor = std::io::Cursor::new(&bytes);

        let exif = exif::Reader::new().read_from_container(&mut cursor).ok();

        match exif
            .as_ref()
            .and_then(|exif| exif.get_field(Tag::Orientation, In::PRIMARY))
            .and_then(|field| field.value.get_uint(0))
        {
            Some(1) => Orientation::NoTransforms,
            Some(2) => Orientation::FlipHorizontal,
            Some(3) => Orientation::Rotate180,
            Some(4) => Orientation::FlipVertical,
            Some(6) => Orientation::Rotate90,
            Some(8) => Orientation::Rotate270,
            _ => Orientation::NoTransforms,
        }
    };

    image.apply_orientation(orientation);

    let width = width.unwrap_or(u32::MAX);
    let height = height.unwrap_or(u32::MAX);

    let resized = image.resize(width, height, FilterType::Lanczos3);

    let mut output = Vec::new();

    resized
        .write_to(
            &mut std::io::Cursor::new(&mut output),
            image::ImageFormat::Jpeg,
        )
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    tokio::fs::create_dir_all(cache_path.parent().unwrap())
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    tokio::fs::write(&cache_path, &output)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(image_response(output))
}
