use async_trait::async_trait;
use sqlx::{query_as, FromRow, PgPool};

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
    pool: PgPool,
}

impl SqlxFactsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct SqlxFact {
    id: i32,
    title: String,
    body: String,
}

impl TryFrom<SqlxFact> for Fact {
    type Error = FactError;

    fn try_from(value: SqlxFact) -> Result<Self, Self::Error> {
        Fact::new(value.id, &value.title, &value.body)
    }
}

impl From<Fact> for SqlxFact {
    fn from(val: Fact) -> Self {
        SqlxFact {
            id: val.id().into(),
            title: val.title().to_owned().into(),
            body: val.body().to_owned().into(),
        }
    }
}

#[async_trait]
impl FactsRepository for SqlxFactsRepository {
    async fn get(&self, id: i32) -> Result<Fact, GetFactError> {
        let result = query_as!(
            SqlxFact,
            r"
SELECT
  id, title, body
FROM facts
WHERE id = $1
        ",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .transpose()
        .ok_or(GetFactError::NoSuchFact { id })?
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
        let result = query_as!(
            SqlxFact,
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
    use sqlx::{query, query_scalar};

    use super::*;

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get(pool: PgPool) {
        let fake = Faker.fake::<Fact>();
        let entity: SqlxFact = fake.clone().into();

        let id = query_scalar!(
            "INSERT INTO facts (title, body) VALUES ($1, $2) RETURNING id",
            entity.title,
            entity.body,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let repo = SqlxFactsRepository::new(pool);

        let result: Fact = repo.get(id).await.unwrap();

        assert_eq!(fake.body(), result.body());
        assert_eq!(fake.title(), result.title());
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_non_existent(pool: PgPool) {
        let repo = SqlxFactsRepository::new(pool);
        let id = Faker.fake();
        let result = repo.get(id).await;

        assert_eq!(result, Err(GetFactError::NoSuchFact { id }));
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_random_from_empty_map(pool: PgPool) {
        let repo = SqlxFactsRepository::new(pool);
        let result = repo.get_random().await;

        assert_eq!(result, Err(GetRandomFactError::Empty));
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_random_from_one_element(pool: PgPool) {
        let fake = Faker.fake::<Fact>();
        let entity: SqlxFact = fake.clone().into();

        query!(
            "INSERT INTO facts (title, body) VALUES ($1, $2)",
            entity.title,
            entity.body,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SqlxFactsRepository::new(pool);

        let result = repo.get_random().await.unwrap();

        assert_eq!(fake.title(), result.title());
        assert_eq!(fake.body(), result.body());
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_random(pool: PgPool) {
        for _ in 0..32 {
            let fake = Faker.fake::<Fact>();
            let entity: SqlxFact = fake.clone().into();

            query!(
                "INSERT INTO facts (title, body) VALUES ($1, $2)",
                entity.title,
                entity.body,
            )
            .execute(&pool)
            .await
            .unwrap();
        }

        let repo = SqlxFactsRepository::new(pool);

        repo.get_random().await.unwrap();
    }
}
