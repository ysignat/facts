use async_trait::async_trait;
use sqlx::{query_as, AnyPool, FromRow};

use super::{
    errors::{GetFactError, GetRandomFactError},
    models::{Fact, FactError},
    FactsRepository,
};

#[derive(Clone)]
pub struct MockedFactsRepository {}

const TITLE: &str = "About smoking";
const BODY: &str = r#"The phrase "smoking kills" is a direct statement about the severe health risks of tobacco use
Smoking is a leading cause of preventable death globally, leading to cancer, heart disease, stroke, and lung diseases like emphysema"#;

#[async_trait]
impl FactsRepository for MockedFactsRepository {
    async fn get(&self, id: i32) -> Result<Fact, GetFactError> {
        Fact::new(id, TITLE, BODY).map_err(|err| GetFactError::UnexpectedError {
            inner: err.to_string(),
        })
    }

    async fn get_random(&self) -> Result<Fact, GetRandomFactError> {
        Fact::new(42, TITLE, BODY).map_err(|err| GetRandomFactError::UnexpectedError {
            inner: err.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct SqlxFactsRepository {
    pool: AnyPool,
}

impl SqlxFactsRepository {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct SqlxEntity {
    id: i32,
    title: String,
    body: String,
}

impl TryFrom<SqlxEntity> for Fact {
    type Error = FactError;

    fn try_from(value: SqlxEntity) -> Result<Self, Self::Error> {
        Fact::new(value.id, &value.title, &value.body)
    }
}

impl From<Fact> for SqlxEntity {
    fn from(val: Fact) -> Self {
        SqlxEntity {
            id: val.id().into(),
            title: val.title().to_owned().into(),
            body: val.body().to_owned().into(),
        }
    }
}

#[async_trait]
impl FactsRepository for SqlxFactsRepository {
    async fn get(&self, id: i32) -> Result<Fact, GetFactError> {
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
        .ok_or(GetFactError::NoSuchEntity { id })?
        .map_err(|err| GetFactError::UnexpectedError {
            inner: err.to_string(),
        })?;

        result
            .try_into()
            .map_err(|err: FactError| GetFactError::UnexpectedError {
                inner: err.to_string(),
            })
    }

    async fn get_random(&self) -> Result<Fact, GetRandomFactError> {
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
        .ok_or(GetRandomFactError::Empty)?
        .map_err(|err| GetRandomFactError::UnexpectedError {
            inner: err.to_string(),
        })?;

        result
            .try_into()
            .map_err(|err: FactError| GetRandomFactError::UnexpectedError {
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
        let fake = Faker.fake::<Fact>();
        let entity: SqlxEntity = fake.clone().into();

        query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
            .bind(entity.id)
            .bind(entity.title)
            .bind(entity.body)
            .execute(&pool)
            .await
            .unwrap();

        let repo = SqlxFactsRepository::new(pool);

        let result: Fact = repo.get(entity.id).await.unwrap();

        assert_eq!(fake, result);
    }

    #[tokio::test]
    async fn get_non_existent() {
        let pool = setup().await;
        let repo = SqlxFactsRepository::new(pool);
        let id = Faker.fake();
        let result = repo.get(id).await;

        assert_eq!(result, Err(GetFactError::NoSuchEntity { id }));
    }

    #[tokio::test]
    async fn get_random_from_empty_map() {
        let pool = setup().await;
        let repo = SqlxFactsRepository::new(pool);
        let result = repo.get_random().await;

        assert_eq!(result, Err(GetRandomFactError::Empty));
    }

    #[tokio::test]
    async fn get_random_from_one_element() {
        let pool = setup().await;
        let fake = Faker.fake::<Fact>();
        let entity: SqlxEntity = fake.clone().into();

        query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
            .bind(entity.id)
            .bind(entity.title)
            .bind(entity.body)
            .execute(&pool)
            .await
            .unwrap();

        let repo = SqlxFactsRepository::new(pool);

        let result = repo.get_random().await.unwrap();

        assert_eq!(fake, result);
    }

    #[tokio::test]
    async fn get_random() {
        let pool = setup().await;
        for _ in 0..32 {
            let fake = Faker.fake::<Fact>();
            let entity: SqlxEntity = fake.clone().into();

            query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
                .bind(entity.id)
                .bind(entity.title)
                .bind(entity.body)
                .execute(&pool)
                .await
                .unwrap();
        }

        let repo = SqlxFactsRepository::new(pool);

        repo.get_random().await.unwrap();
    }
}
