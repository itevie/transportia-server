use std::sync::Arc;
use crate::db::Database;
use crate::db::resource::checklist::checklist_def::{Checklist, ChecklistType};
use crate::http::errors::ApiError;

pub async fn check_checklist_obj(db: Arc<Database>, checklist: Checklist, obj_id: String) -> Result<(), ApiError> {
    match checklist.checklist_type {
        ChecklistType::Service => {
            db.tx_lines
                .get(obj_id.as_str())
                .await
                .map_err(|x| ApiError::ValidationError(format!(
                    "Given checklist item's obj does not match the checklist's type: {}", x
                )))?
                .ok_or(ApiError::NotFound("Checklist item not found".into()))?;
        }
        ChecklistType::Vehicle => {
            let vehicle_id = obj_id.parse::<i64>()
                .map_err(|x| ApiError::ValidationError(format!(
                    "Given checklist item's obj does not match the checklist's type: {}", x
                )))?;

            db.vehicles
                .get(vehicle_id)
                .await
                .map_err(|x| ApiError::ValidationError(format!(
                    "Given checklist item's obj does not match the checklist's type: {}", x
                )))?
                .ok_or(ApiError::NotFound("Checklist item not found".into()))?;
        }
    }

    Ok(())
}