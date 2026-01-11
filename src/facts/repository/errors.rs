use thiserror::Error;

use super::models::FactId;

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum GetFactError {
    #[error("Fact with id '{id:?}' doesn't exist in our records")]
    NoSuchFact { id: FactId },
    #[error("Something weird occured while retrieving the fact: {inner}")]
    UnexpectedError { inner: String },
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum GetRandomFactError {
    #[error("Collection is empty, nothing to choose")]
    Empty,
    #[error("Something weird occured while retrieving the random fact: {inner}")]
    UnexpectedError { inner: String },
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum CreateFactError {
    #[error("Something weird occured while creating the fact: {inner}")]
    UnexpectedError { inner: String },
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum DeleteFactError {
    #[error("Fact with id '{id:?}' doesn't exist in our records")]
    NoSuchFact { id: FactId },
    #[error("Something weird occured while deleting the fact: {inner}")]
    UnexpectedError { inner: String },
}
