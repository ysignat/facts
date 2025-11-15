use std::collections::HashMap;

use async_trait::async_trait;
use rand::{rng, seq::IndexedRandom};

use crate::facts::dao::{
    dtos::Entity,
    errors::{GetError, GetRandomError},
    Dao,
};

#[derive(Clone)]
pub struct HashMapDao(HashMap<u64, Entity>);

impl HashMapDao {
    pub fn new(hash_map: HashMap<u64, Entity>) -> Self {
        Self(hash_map)
    }
}

#[async_trait]
impl Dao for HashMapDao {
    async fn get(&self, id: u64) -> Result<Entity, GetError> {
        self.0
            .get(&id)
            .map(ToOwned::to_owned)
            .ok_or(GetError::NoSuchEntity { id })
    }

    async fn get_random(&self) -> Result<Entity, GetRandomError> {
        let id = self
            .0
            .keys()
            .collect::<Vec<&u64>>()
            .choose(&mut rng())
            .ok_or(GetRandomError::Empty)?
            .to_owned();

        Ok(self.0.get(id).unwrap().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    #[tokio::test]
    async fn get() {
        let mut dao = HashMapDao::new(HashMap::new());
        let entity = Faker.fake::<Entity>();

        dao.0.insert(entity.id(), entity.clone());

        let result = dao.get(entity.id()).await.unwrap();

        assert_eq!(entity, result);
    }

    #[tokio::test]
    async fn get_non_existent() {
        let dao = HashMapDao::new(HashMap::new());
        let id = Faker.fake();
        let result = dao.get(id).await;

        assert_eq!(result, Err(GetError::NoSuchEntity { id }));
    }

    #[tokio::test]
    async fn get_random_from_empty_map() {
        let dao = HashMapDao::new(HashMap::new());
        let result = dao.get_random().await;

        assert_eq!(result, Err(GetRandomError::Empty));
    }

    #[tokio::test]
    async fn get_random_from_one_element() {
        let mut dao = HashMapDao::new(HashMap::new());
        let entity = Faker.fake::<Entity>();

        dao.0.insert(entity.id(), entity.clone());

        let result = dao.get_random().await.unwrap();

        assert_eq!(entity, result);
    }

    #[tokio::test]
    async fn get_random() {
        let mut dao = HashMapDao::new(HashMap::new());
        let entity = Faker.fake::<Entity>();

        for _ in 0..32 {
            dao.0.insert(entity.id(), entity.clone());
        }

        dao.get_random().await.unwrap();
    }
}
