use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::facts::repository::{GetFactError, GetRandomFactError};

pub struct AppError {
    pub status_code: StatusCode,
    pub details: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status_code, self.details).into_response()
    }
}

impl From<GetFactError> for AppError {
    fn from(value: GetFactError) -> Self {
        let status_code = match value {
            GetFactError::NoSuchEntity { id: _ } => StatusCode::NOT_FOUND,
            GetFactError::UnexpectedError { inner: _ } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self {
            status_code,
            details: value.to_string(),
        }
    }
}

impl From<GetRandomFactError> for AppError {
    fn from(value: GetRandomFactError) -> Self {
        let status_code = match value {
            GetRandomFactError::Empty => StatusCode::NOT_FOUND,
            GetRandomFactError::UnexpectedError { inner: _ } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self {
            status_code,
            details: value.to_string(),
        }
    }
}
