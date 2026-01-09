use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum GetError {
    #[error("Entity with id '{id:?}' doesn't exist in our records")]
    NoSuchEntity { id: i64 },
    #[error("Something weird occured while retrieving the entity: {inner}")]
    UnexpectedError { inner: String },
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum GetRandomError {
    #[error("Collection is empty, nothing to choose")]
    Empty,
    #[error("Something weird occured while retrieving the entity: {inner}")]
    UnexpectedError { inner: String },
}
