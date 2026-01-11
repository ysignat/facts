use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::facts::repository::{
    CreateFactError,
    CreateFactRequestError,
    DeleteFactError,
    FactIdError,
    GetFactError,
    GetRandomFactError,
};

pub struct AppError {
    pub status_code: StatusCode,
    pub details: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status_code, self.details).into_response()
    }
}

impl From<FactIdError> for AppError {
    fn from(value: FactIdError) -> Self {
        Self {
            status_code: StatusCode::UNPROCESSABLE_ENTITY,
            details: value.to_string(),
        }
    }
}

impl From<GetFactError> for AppError {
    fn from(value: GetFactError) -> Self {
        let status_code = match value {
            GetFactError::NoSuchFact { id: _ } => StatusCode::NOT_FOUND,
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

impl From<CreateFactError> for AppError {
    fn from(value: CreateFactError) -> Self {
        let status_code = match value {
            CreateFactError::UnexpectedError { inner: _ } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self {
            status_code,
            details: value.to_string(),
        }
    }
}

impl From<DeleteFactError> for AppError {
    fn from(value: DeleteFactError) -> Self {
        let status_code = match value {
            DeleteFactError::NoSuchFact { id: _ } => StatusCode::NOT_FOUND,
            DeleteFactError::UnexpectedError { inner: _ } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self {
            status_code,
            details: value.to_string(),
        }
    }
}

impl From<CreateFactRequestError> for AppError {
    fn from(value: CreateFactRequestError) -> Self {
        Self {
            status_code: StatusCode::UNPROCESSABLE_ENTITY,
            details: value.to_string(),
        }
    }
}
