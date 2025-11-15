use async_trait::async_trait;
pub use dtos::Entity;
pub use errors::{GetError, GetRandomError};
pub use impls::{HashMapDao, MockedDao};

mod dtos;
mod errors;
mod impls;

#[async_trait]
pub trait Dao: Send + Sync {
    async fn get(&self, id: u64) -> Result<Entity, GetError>;
    async fn get_random(&self) -> Result<Entity, GetRandomError>;
}
