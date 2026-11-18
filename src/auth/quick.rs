use std::sync::Arc;
use crate::db::Database;
use crate::db::resource::checklist::checklist_def::RouteChecklist;
use crate::db::resource::favourite::favourite_group_def::FavouritesGroup;
use crate::db::resource::photo::photo_def::Photo;
use crate::http::errors::ApiError;

pub async fn check_route_checklist(db: Arc<Database>, checklist_id: i64, user_id: i64) -> Result<RouteChecklist, ApiError> {
    let checklist = db
        .route_checklists
        .get(checklist_id)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(checklist_id.to_string()))?;

    if checklist.owner_id != user_id {
        return Err(ApiError::Unauthorized(
            "You do not own this checklist".to_string(),
        ));
    }
    
    Ok(checklist)
}

pub async fn check_favourites_group(db: Arc<Database>, favourites_group_id: i64, user_id: i64) -> Result<FavouritesGroup, ApiError> {
    let group = db
        .favourites_groups
        .get(favourites_group_id)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(favourites_group_id.to_string()))?;

    if group.owner_id != user_id {
        return Err(ApiError::Unauthorized(
            "You do not own this favourites group".to_string(),
        ));
    }

    Ok(group)
}

pub async fn check_photo(db: Arc<Database>, photo_id: i64, user_id: i64) -> Result<Photo, ApiError> {
    let photo = db
        .photos
        .get(photo_id)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(photo_id.to_string()))?;

    if photo.author_id != user_id {
        return Err(ApiError::Unauthorized(
            "You do not own this photo".to_string(),
        ));
    }

    Ok(photo)
}

pub async fn is_trusted_user(db: Arc<Database>, user_id: i64) -> Result<bool, ApiError> {
    let user = db
        .users
        .get(user_id, false)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
    
    match user {
        Some(val) => if !val.trusted_user {
            Err(ApiError::Unauthorized("You must be a trusted user to do this!".to_string()))
        } else {
            Ok(true)
        },
        None => Err(ApiError::Unauthorized("You must be a trusted user to do this!".to_string()))
    }
}

pub async fn is_admin(db: Arc<Database>, user_id: i64) -> Result<bool, ApiError> {
    let user = db
        .users
        .get(user_id, false)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    match user {
        Some(val) => if !val.admin {
            Err(ApiError::Unauthorized("You must be an admin to do this!".to_string()))
        } else {
            Ok(true)
        },
        None => Err(ApiError::Unauthorized("You must be an admin to do this!".to_string()))
    }
}