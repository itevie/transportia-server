use std::ops::RangeInclusive;
use serde::Serialize;
use crate::http::errors::ApiError;
use crate::validation::regex::USERNAME_REGEX;

#[derive(Debug, Clone, Serialize)]
pub enum FieldValidationType {
    Username,
    DisplayName,
    Bio,
}

fn range_to_string(range: &RangeInclusive<usize>) -> String {
    format!("{}..={}", range.start(), range.end())
}

pub fn check_field_valid(field: String, field_type: FieldValidationType) -> Result<String, ApiError> {
    let length = match field_type {
        FieldValidationType::Username => Some(3..=16),
        FieldValidationType::DisplayName => Some(0..=32),
        FieldValidationType::Bio => Some(0..=1000),
    };

    if let Some(length) = length {
        if !length.contains(&field.len()) {
            return Err(ApiError::ValidationError(format!("{:?} is invalid, invalid length: {}", field_type, range_to_string(&length))));
        }
    }

    let regex = match field_type {
        FieldValidationType::Username => Some(&*USERNAME_REGEX),
        _ => None
    };

    if let Some(regex) = regex {
        if !regex.is_match(&field) {
            return Err(ApiError::ValidationError(format!(
                "{:?} must match the following pattern: {}",
                field_type,
                regex.as_str()
            )));
        }
    }

    Ok(field.clone())
}

pub fn check_field_valid_option(field: Option<String>, field_type: FieldValidationType) -> Result<Option<String>, ApiError> {
    let Some(field) = field else {
        return Ok(None);
    };

    check_field_valid(field, field_type).map(|x| Some(x))
}