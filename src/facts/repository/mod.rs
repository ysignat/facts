use async_trait::async_trait;
pub use errors::{CreateFactError, DeleteFactError, GetFactError, GetRandomFactError};
pub use impls::{MockedFactsRepository, SqlxFactsRepository};
pub use models::{
    CreateFactRequest,
    CreateFactRequestError,
    Fact,
    FactBody,
    FactId,
    FactIdError,
    FactTitle,
};

mod errors;
mod impls;
mod models;

#[async_trait]
pub trait FactsRepository: Send + Sync {
    async fn get(&self, id: FactId) -> Result<Fact, GetFactError>;
    async fn get_random(&self) -> Result<Fact, GetRandomFactError>;
    async fn create(&self, data: &CreateFactRequest) -> Result<Fact, CreateFactError>;
    async fn delete(&self, id: FactId) -> Result<(), DeleteFactError>;
}
