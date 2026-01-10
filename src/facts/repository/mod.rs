use async_trait::async_trait;
pub use errors::{GetFactError, GetRandomFactError};
pub use impls::{MockedFactsRepository, SqlxFactsRepository};
pub use models::Fact;

mod errors;
mod impls;
mod models;

#[async_trait]
pub trait FactsRepository: Send + Sync {
    async fn get(&self, id: i32) -> Result<Fact, GetFactError>;
    async fn get_random(&self) -> Result<Fact, GetRandomFactError>;
}
