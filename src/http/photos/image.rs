use std::io::Cursor;
use std::path::{Path, PathBuf};

use axum::extract::{Path as PathExtractor, Query, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use image::imageops::FilterType;
use image::metadata::Orientation;
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::extractors::ratelimit::{MediaRatelimit, RateLimit};
use crate::http::errors::ApiError;
use crate::http::HttpState;
use crate::util::exif::get_photo_orientation;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PhotoImageQuery {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

const CACHE_DIR: &str = "data/photo_cache";
const CDN_URL: &str = "https://transportia-cdn.dawn.rest";

fn image_response(bytes: Vec<u8>, content_type: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable",
        )
        .body(bytes.into())
        .unwrap()
}

async fn fetch_original_image(uploaded_id: &str) -> Result<Vec<u8>, ApiError> {
    let url = format!("{CDN_URL}/{uploaded_id}");

    let response = reqwest::get(&url)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::GenericServerError(format!(
            "Failed to fetch image: HTTP {}",
            response.status()
        )));
    }

    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|err| ApiError::GenericServerError(err.to_string()))
}

fn decode_image(bytes: &[u8]) -> Result<DynamicImage, ApiError> {
    image::load_from_memory(bytes)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))
}

fn encode_image(image: DynamicImage, format: ImageFormat) -> Result<Vec<u8>, ApiError> {
    let mut output = Cursor::new(Vec::new());

    image
        .write_to(&mut output, format)
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(output.into_inner())
}

async fn read_cache(path: &Path) -> Result<Option<Vec<u8>>, ApiError> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(ApiError::GenericServerError(err.to_string())),
    }
}

async fn write_cache(path: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    tokio::fs::write(path, bytes)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))
}

fn resized_cache_path(id: i64, width: Option<u32>, height: Option<u32>) -> PathBuf {
    let name = match (width, height) {
        (Some(w), Some(h)) => format!("{id}_{w}x{h}.jpg"),
        (Some(w), None) => format!("{id}_w{w}.jpg"),
        (None, Some(h)) => format!("{id}_h{h}.jpg"),
        (None, None) => unreachable!(),
    };

    PathBuf::from(CACHE_DIR).join(name)
}

fn original_cache_path(id: i64, orientation: Orientation) -> PathBuf {
    PathBuf::from(CACHE_DIR).join(format!("{id}_original_{orientation:?}.webp"))
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
            description = "Maximum image width."
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
            description = "Photo image",
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
    _ratelimit: RateLimit<MediaRatelimit>,
    PathExtractor(id): PathExtractor<i64>,
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

    let orientation = get_photo_orientation(photo.exif.as_ref());

    // No dimensions: return the original WebP.
    if width.is_none() && height.is_none() {
        // Only transformed originals need a separate cached copy.
        if orientation != Orientation::NoTransforms {
            let cache_path = original_cache_path(id, orientation);

            if let Some(bytes) = read_cache(&cache_path).await? {
                return Ok(image_response(bytes, "image/webp"));
            }

            let bytes = fetch_original_image(&photo.uploaded_id).await?;

            let mut image = decode_image(&bytes)?;
            image.apply_orientation(orientation);

            let output = encode_image(image, ImageFormat::WebP)?;

            write_cache(&cache_path, &output).await?;

            return Ok(image_response(output, "image/webp"));
        }

        // No transformation needed; return the original bytes untouched.
        let bytes = fetch_original_image(&photo.uploaded_id).await?;

        return Ok(image_response(bytes, "image/webp"));
    }

    // Resized images are cached as JPEG.
    let cache_path = resized_cache_path(id, width, height);

    if let Some(bytes) = read_cache(&cache_path).await? {
        return Ok(image_response(bytes, "image/jpeg"));
    }

    // Fetch, decode and orient the original image.
    let bytes = fetch_original_image(&photo.uploaded_id).await?;

    let mut image = decode_image(&bytes)?;
    image.apply_orientation(orientation);

    // Resize while preserving the aspect ratio.
    let resized = image.resize(
        width.unwrap_or(u32::MAX),
        height.unwrap_or(u32::MAX),
        FilterType::Lanczos3,
    );

    let output = encode_image(resized, ImageFormat::Jpeg)?;

    write_cache(&cache_path, &output).await?;

    Ok(image_response(output, "image/jpeg"))
}