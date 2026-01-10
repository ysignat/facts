use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum GetFactError {
    #[error("Entity with id '{id:?}' doesn't exist in our records")]
    NoSuchEntity { id: i32 },
    #[error("Something weird occured while retrieving the entity: {inner}")]
    UnexpectedError { inner: String },
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum GetRandomFactError {
    #[error("Collection is empty, nothing to choose")]
    Empty,
    #[error("Something weird occured while retrieving the entity: {inner}")]
    UnexpectedError { inner: String },
}
