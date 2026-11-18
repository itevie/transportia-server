use aws_smithy_types::byte_stream::ByteStream;
use axum::{extract::Multipart, Json};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::resource::photo::photo::Photos;
use crate::db::resource::photo::photo_def::{Photo, PhotoFilledOut, PhotoType};
use crate::extractors::authenticator::AuthUser;
use crate::http::errors::ApiError;
use crate::http::HttpState;
use crate::util::image::compress_image;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct UploadDetails {
    pub photo_type: PhotoType
}

#[utoipa::path(
    post,
    tag = "Photos",
    path = "/photos/upload",
    request_body(
        content = inline(Object),
        content_type = "multipart/form-data",
        description = "One or more JPEG or PNG images to upload."
    ),
    params(UploadDetails),
    summary = "Upload a photo",
    responses(
        (
            status = 200,
            description = "Images uploaded successfully",
            body = [PhotoFilledOut]
        ),
        (
            status = 400,
            description = "Uploaded file is not a JPEG or PNG image"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn upload_image(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    Query(details): Query<UploadDetails>,
    mut multipart: Multipart,
) -> Result<Response, ApiError> {
    let mut photos: Vec<Photo> = vec![];
    
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?
    {
        let file_name = field.file_name().unwrap_or("").to_string();
        let content_type = field.content_type().unwrap_or("").to_string();

        if content_type != "image/jpeg"
            && content_type != "image/png"
            && content_type != "image/webp"
        {
            return Err(ApiError::UploadedNonImage);
        }

        let data = field
            .bytes()
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        let data = compress_image(&data)
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        tokio::fs::write(
            format!("/tmp/{file_name}"),
            &data,
        )
            .await
            .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        let key = format!(
            "photos/{}/{}",
            user_id,
            uuid::Uuid::new_v4()
        );

        // Upload to R2
        state.s3
            .put_object()
            .bucket(std::env::var("R2_BUCKET").unwrap())
            .key(&key)
            .body(ByteStream::from(data))
            .content_type("image/webp")
            .send()
            .await
            .map_err(|err| {
                ApiError::GenericServerError(err.to_string())
            })?;

        let photo = state.db.photos.create(&Photo {
            id: 0,
            author_id: user_id,
            photo_type: details.photo_type,
            obj_details_id: None,
            comment_thread: None,
            file_name,
            uploaded_id: key,
            title: "".to_string(),
            description: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }).await.map_err(|err| ApiError::GenericServerError(err.to_string()))?;

        photos.push(photo);
    }

    let filled_out = Photos::fill_out(state.db.clone(), photos, Some(user_id))
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok((StatusCode::OK, Json(filled_out)).into_response())
}
