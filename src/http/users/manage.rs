use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use crate::auth::quick::check_photo;
use crate::db::resource::photo::photo_def::PhotoUpdate;
use crate::db::resource::user::def::UserUpdate;
use crate::extractors::authenticator::AuthUser;
use crate::extractors::ratelimit::{AuthenticationRatelimit, RateLimit, UpdateRatelimit};
use crate::http::errors::ApiError;
use crate::http::HttpState;
use crate::validation::fields::{check_field_valid, check_field_valid_option, FieldValidationType};

#[utoipa::path(
    patch,
    path = "/users/{id}",
    tag = "Users",
    summary = "Update your user account",
    request_body = UserUpdate,
    responses(
        (
            status = 204,
            description = "Success",
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
pub async fn update_user(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<UpdateRatelimit>,
    AuthUser { user_id }: AuthUser,
    Json(update): Json<UserUpdate>,
) -> Result<StatusCode, ApiError> {
    let mut user = state
        .db
        .users
        .get(user_id, false)
        .await?
        .ok_or(ApiError::NotFound("User not found".to_string()))?;

    if let Some(username) = update.username {
        user.username = check_field_valid(username, FieldValidationType::Username)?;
    }

    if let Some(display_name) = update.display_name {
        user.display_name = check_field_valid_option(display_name, FieldValidationType::DisplayName)?;
    }

    if let Some(bio) = update.bio {
        user.bio = check_field_valid_option(bio, FieldValidationType::Bio)?;
    }
    
    if let Some(public_f) = update.public_favourites {
        user.public_favourites = public_f;
    }
    
    if let Some(public_m) = update.public_models {
        user.public_models = public_m;
    }

    state
        .db
        .users
        .update(&user)
        .await
        .map_err(|err| ApiError::GenericServerError(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}