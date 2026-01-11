use async_trait::async_trait;
use sqlx::{query_as, query_scalar, FromRow, PgPool};

use super::{
    errors::{GetFactError, GetRandomFactError},
    models::{Fact, FactBody, FactError, FactId, FactTitle},
    CreateFactError,
    CreateFactRequest,
    DeleteFactError,
    FactsRepository,
};

#[derive(Clone)]
pub struct MockedFactsRepository {}

const TITLE: &str = "About smoking";
const BODY: &str = r#"The phrase "smoking kills" is a direct statement about the severe health risks of tobacco use
Smoking is a leading cause of preventable death globally, leading to cancer, heart disease, stroke, and lung diseases like emphysema"#;

#[async_trait]
impl FactsRepository for MockedFactsRepository {
    async fn get(&self, id: FactId) -> Result<Fact, GetFactError> {
        Ok(Fact::new(
            id,
            &FactTitle::new(TITLE).map_err(|err| GetFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
            &FactBody::new(BODY).map_err(|err| GetFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
        ))
    }

    async fn get_random(&self) -> Result<Fact, GetRandomFactError> {
        Ok(Fact::new(
            FactId::new(42).map_err(|err| GetRandomFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
            &FactTitle::new(TITLE).map_err(|err| GetRandomFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
            &FactBody::new(BODY).map_err(|err| GetRandomFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
        ))
    }

    async fn create(&self, _: &CreateFactRequest) -> Result<Fact, CreateFactError> {
        Ok(Fact::new(
            FactId::new(43).map_err(|err| CreateFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
            &FactTitle::new(TITLE).map_err(|err| CreateFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
            &FactBody::new(BODY).map_err(|err| CreateFactError::UnexpectedError {
                inner: err.to_string(),
            })?,
        ))
    }

    async fn delete(&self, id: FactId) -> Result<(), DeleteFactError> {
        let err = DeleteFactError::UnexpectedError {
            inner: "This should never happen".to_owned(),
        };

        if id.eq(
            &FactId::new(44).map_err(|_| DeleteFactError::UnexpectedError {
                inner: "This should never happen".to_owned(),
            })?,
        ) {
            Err(err)
        } else if id.eq(
            &FactId::new(45).map_err(|_| DeleteFactError::UnexpectedError {
                inner: "This should never happen".to_owned(),
            })?,
        ) {
            Err(DeleteFactError::NoSuchFact { id })
        } else {
            Ok(())
        }
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
        Ok(Fact::new(
            FactId::new(value.id)?,
            &FactTitle::new(&value.title)?,
            &FactBody::new(&value.body)?,
        ))
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
    async fn get(&self, id: FactId) -> Result<Fact, GetFactError> {
        let result = query_as!(
            SqlxFact,
            r"
SELECT
  id, title, body
FROM facts
WHERE id = $1
        ",
            i32::from(id)
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

    async fn create(&self, data: &CreateFactRequest) -> Result<Fact, CreateFactError> {
        let result = query_as!(
            SqlxFact,
            r"
INSERT INTO facts (title, body)
VALUES ($1, $2)
RETURNING id, title, body
        ",
            String::from(data.title().to_owned()),
            String::from(data.body().to_owned()),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| CreateFactError::UnexpectedError {
            inner: err.to_string(),
        })?;

        result
            .try_into()
            .map_err(|err: FactError| CreateFactError::UnexpectedError {
                inner: err.to_string(),
            })
    }

    async fn delete(&self, id: FactId) -> Result<(), DeleteFactError> {
        query_scalar!(
            r"
DELETE FROM facts
WHERE id = $1
RETURNING id
        ",
            i32::from(id)
        )
        .fetch_optional(&self.pool)
        .await
        .transpose()
        .ok_or(DeleteFactError::NoSuchFact { id })?
        .map_err(|err| DeleteFactError::UnexpectedError {
            inner: err.to_string(),
        })?;

        Ok(())
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

        let result: Fact = repo.get(FactId::new(id).unwrap()).await.unwrap();

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
    async fn get_random_from_empty(pool: PgPool) {
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
    async fn get_random_from_one_corrupted_element(pool: PgPool) {
        let fake = Faker.fake::<Fact>();
        let entity: SqlxFact = fake.clone().into();

        query!(
            "INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)",
            0,
            entity.title,
            entity.body,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SqlxFactsRepository::new(pool);

        assert!(matches!(
            repo.get_random().await,
            Err(GetRandomFactError::UnexpectedError { inner: _ })
        ));
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

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn create(pool: PgPool) {
        let repo = SqlxFactsRepository::new(pool);
        let mut id: i32 = 0;

        for _ in 0..32 {
            let fake = Faker.fake::<CreateFactRequest>();
            let fact = repo.create(&fake).await.unwrap();

            if id.ne(&0) {
                assert_eq!(i32::from(fact.id()), id + 1);
                id += 1;
            } else {
                id = i32::from(fact.id());
            }
        }
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn delete_non_existent(pool: PgPool) {
        let repo = SqlxFactsRepository::new(pool);
        let id = Faker.fake();
        let result = repo.delete(id).await;

        assert_eq!(result, Err(DeleteFactError::NoSuchFact { id }));
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn double_delete(pool: PgPool) {
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

        repo.delete(FactId::new(id).unwrap()).await.unwrap();

        assert!(matches!(
            repo.delete(FactId::new(id).unwrap()).await,
            Err(DeleteFactError::NoSuchFact { id: _ })
        ));
    }
}
