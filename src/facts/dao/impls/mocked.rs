use async_trait::async_trait;

use crate::facts::dao::{
    dtos::{Builder, Entity},
    errors::{GetError, GetRandomError},
    Dao,
};

#[derive(Clone)]
pub struct MockedDao {}

const TITLE: &str = "About smoking";
const BODY: &str = r#"The phrase "smoking kills" is a direct statement about the severe health risks of tobacco use
Smoking is a leading cause of preventable death globally, leading to cancer, heart disease, stroke, and lung diseases like emphysema"#;

#[async_trait]
impl Dao for MockedDao {
    async fn get(&self, id: i64) -> Result<Entity, GetError> {
        Builder::new()
            .id(id)
            .title(TITLE.to_owned())
            .body(BODY.to_owned())
            .build()
            .map_err(|err| GetError::UnexpectedError {
                inner: err.to_string(),
            })
    }

    async fn get_random(&self) -> Result<Entity, GetRandomError> {
        Builder::new()
            .id(42)
            .title(TITLE.to_owned())
            .body(BODY.to_owned())
            .build()
            .map_err(|err| GetRandomError::UnexpectedError {
                inner: err.to_string(),
            })
    }
}
