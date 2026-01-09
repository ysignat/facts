use async_trait::async_trait;
use sqlx::{query_as, AnyPool, FromRow};

use crate::facts::dao::{
    dtos::{Builder, BuilderError, Entity},
    errors::{GetError, GetRandomError},
    Dao,
};

#[derive(Clone)]
pub struct SqlxDao {
    pool: AnyPool,
}

impl SqlxDao {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct SqlxEntity {
    id: i64,
    title: String,
    body: String,
}

impl TryFrom<SqlxEntity> for Entity {
    type Error = BuilderError;

    fn try_from(value: SqlxEntity) -> Result<Self, Self::Error> {
        Builder::new()
            .id(value.id)
            .title(value.title)
            .body(value.body)
            .build()
    }
}

#[async_trait]
impl Dao for SqlxDao {
    async fn get(&self, id: i64) -> Result<Entity, GetError> {
        let result: SqlxEntity = query_as(
            r"
SELECT
  id, title, body
FROM facts
WHERE id = $1
        ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .transpose()
        .ok_or(GetError::NoSuchEntity { id })?
        .map_err(|err| GetError::UnexpectedError {
            inner: err.to_string(),
        })?;

        result
            .try_into()
            .map_err(|err: BuilderError| GetError::UnexpectedError {
                inner: err.to_string(),
            })
    }

    async fn get_random(&self) -> Result<Entity, GetRandomError> {
        let result: SqlxEntity = query_as(
            r"
SELECT
  id, title, body
FROM facts
ORDER BY random()
LIMIT 1
        ",
        )
        .fetch_optional(&self.pool)
        .await
        .transpose()
        .ok_or(GetRandomError::Empty)?
        .map_err(|err| GetRandomError::UnexpectedError {
            inner: err.to_string(),
        })?;

        result
            .try_into()
            .map_err(|err: BuilderError| GetRandomError::UnexpectedError {
                inner: err.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use sqlx::{
        any::{install_default_drivers, AnyPoolOptions},
        migrate::Migrator,
        query,
        AnyPool,
    };

    use super::*;

    static MIGRATOR: Migrator = sqlx::migrate!("./src/facts/migrations");

    async fn setup() -> AnyPool {
        install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        MIGRATOR.run(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn get() {
        let pool = setup().await;
        let entity = Faker.fake::<Entity>();

        query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
            .bind(entity.id())
            .bind(entity.title())
            .bind(entity.body())
            .execute(&pool)
            .await
            .unwrap();

        let dao = SqlxDao::new(pool);

        let result = dao.get(entity.id()).await.unwrap();

        assert_eq!(entity, result);
    }

    #[tokio::test]
    async fn get_non_existent() {
        let pool = setup().await;
        let dao = SqlxDao::new(pool);
        let id = Faker.fake();
        let result = dao.get(id).await;

        assert_eq!(result, Err(GetError::NoSuchEntity { id }));
    }

    #[tokio::test]
    async fn get_random_from_empty_map() {
        let pool = setup().await;
        let dao = SqlxDao::new(pool);
        let result = dao.get_random().await;

        assert_eq!(result, Err(GetRandomError::Empty));
    }

    #[tokio::test]
    async fn get_random_from_one_element() {
        let pool = setup().await;
        let entity = Faker.fake::<Entity>();

        query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
            .bind(entity.id())
            .bind(entity.title())
            .bind(entity.body())
            .execute(&pool)
            .await
            .unwrap();

        let dao = SqlxDao::new(pool);

        let result = dao.get_random().await.unwrap();

        assert_eq!(entity, result);
    }

    #[tokio::test]
    async fn get_random() {
        let pool = setup().await;
        for _ in 0..32 {
            let entity = Faker.fake::<Entity>();

            query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
                .bind(entity.id())
                .bind(entity.title())
                .bind(entity.body())
                .execute(&pool)
                .await
                .unwrap();
        }

        let dao = SqlxDao::new(pool);

        dao.get_random().await.unwrap();
    }
}
