use std::backtrace::Backtrace;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub enum ApiError {
    Database(String),
    InvalidCredentials,
    EmailExists,
    EmailDoesNotExist,
    UsernameExists,
    UserNotFound,
    InvalidUsername,
    GenericServerError(String),
    BadRequest(String),
    NotFound(String),
    UploadedNonImage,
    Unauthorized(String),
    ValidationError(String),
}

pub type ApiErrorResult<T> = Result<T, ApiError>;

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound("Database record not found".into()),
            error => Self::Database(error.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let backtrace = Backtrace::force_capture();

        println!(
            "{:#?}",
            backtrace
        );

        match self {
            ApiError::Database(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error,
            )
                .into_response(),

            ApiError::BadRequest(error) => (
                StatusCode::BAD_REQUEST,
                error,
            )
                .into_response(),

            ApiError::ValidationError(error) => (
                StatusCode::BAD_REQUEST,
                format!("Validation Error: {}", error),
            )
                .into_response(),

            ApiError::Unauthorized(error) => (
                StatusCode::UNAUTHORIZED,
                error,
            )
                .into_response(),

            ApiError::NotFound(error) => (
                StatusCode::NOT_FOUND,
                error,
            )
                .into_response(),

            ApiError::GenericServerError(error) => {
                println!("Generic Server Error: {}", error);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Generic server error",
                )
                    .into_response();
            }

            ApiError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid username or password",
            )
                .into_response(),

            ApiError::UsernameExists => (
                StatusCode::UNAUTHORIZED,
                "Username already exists",
            )
                .into_response(),

            ApiError::EmailExists => (
                StatusCode::UNAUTHORIZED,
                "Email already exists",
            )
                .into_response(),

            ApiError::EmailDoesNotExist => (
                StatusCode::NOT_FOUND,
                "Email doesn't exist",
            )
                .into_response(),

            ApiError::UserNotFound => (
                StatusCode::UNAUTHORIZED,
                "User not found",
            )
                .into_response(),

            ApiError::InvalidUsername => (
                StatusCode::UNAUTHORIZED,
                "Invalid username. Must only contain letters, numbers, . or _, and must be 3-20 characters.",
            )
                .into_response(),

            ApiError::UploadedNonImage => (
                StatusCode::UNAUTHORIZED,
                "Only images can be uploaded.",
            )
                .into_response(),
        }
    }
}
