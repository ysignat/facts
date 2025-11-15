use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::facts::dao::{GetError, GetRandomError};

pub struct AppError {
    pub status_code: StatusCode,
    pub details: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status_code, self.details).into_response()
    }
}

impl From<GetError> for AppError {
    fn from(value: GetError) -> Self {
        let status_code = match value {
            GetError::NoSuchEntity { id: _ } => StatusCode::NOT_FOUND,
        };

        Self {
            status_code,
            details: value.to_string(),
        }
    }
}

impl From<GetRandomError> for AppError {
    fn from(value: GetRandomError) -> Self {
        let status_code = match value {
            GetRandomError::Empty => StatusCode::NOT_FOUND,
        };

        Self {
            status_code,
            details: value.to_string(),
        }
    }
}
