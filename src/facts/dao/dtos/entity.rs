#[cfg(test)]
use fake::{faker::lorem::en::Sentence, Dummy};
use thiserror::Error;

#[derive(Clone)]
#[cfg_attr(test, derive(Dummy, Eq, PartialEq, Debug))]
pub struct Entity {
    id: u64,
    #[cfg_attr(test, dummy(faker = "Sentence(2..3)"))]
    title: String,
    #[cfg_attr(test, dummy(faker = "Sentence(5..10)"))]
    body: String,
}

impl Entity {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Builder {
    id: Option<u64>,
    title: Option<String>,
    body: Option<String>,
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum BuilderError {
    #[error("Id was not set in builder")]
    IdNotSet,
    #[error("Id could not be zero")]
    IdLessOrEqualsZero,
    #[error("Title was not set in builder")]
    TitleNotSet,
    #[error("Title is very long: {length:?} chars")]
    TitleTooLong { length: usize },
    #[error("Empty title is not allowed")]
    TitleIsEmpty,
    #[error("Body was not set in builder")]
    BodyNotSet,
    #[error("Title is very long: {length:?} chars")]
    BodyTooLong { length: usize },
    #[error("Empty title is not allowed")]
    BodyIsEmpty,
}

impl Builder {
    const MAX_TITLE_LENGTH: usize = 64;
    const MAX_BODY_LENGTH: usize = 2048;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Result<Entity, BuilderError> {
        let id = self.id.ok_or(BuilderError::IdNotSet)?;
        let title = self.title.ok_or(BuilderError::TitleNotSet)?;
        let body = self.body.ok_or(BuilderError::BodyNotSet)?;

        if id.le(&0) {
            return Err(BuilderError::IdLessOrEqualsZero);
        }

        if title.is_empty() {
            return Err(BuilderError::TitleIsEmpty);
        }
        if title.len().gt(&Self::MAX_TITLE_LENGTH) {
            return Err(BuilderError::TitleTooLong {
                length: title.len(),
            });
        }

        if body.is_empty() {
            return Err(BuilderError::BodyIsEmpty);
        }
        if body.len().gt(&Self::MAX_BODY_LENGTH) {
            return Err(BuilderError::BodyTooLong { length: body.len() });
        }

        Ok(Entity { id, title, body })
    }
}

#[cfg(test)]
mod tests {
    use fake::{faker::lorem::en::Sentence, Fake, Faker};

    use super::*;

    #[test]
    fn default_builder() {
        let builder = Builder::default();

        assert_eq!(builder.id, None);
        assert_eq!(builder.body, None);
        assert_eq!(builder.title, None);
    }

    #[test]
    fn id_not_set() {
        let title = Sentence(2..3).fake();
        let body = Sentence(5..10).fake();

        let builder = Builder::default();
        let builder_err = builder.title(title).body(body).build();

        assert_eq!(builder_err, Err(BuilderError::IdNotSet));
    }

    #[test]
    fn title_not_set() {
        let id = Faker.fake();
        let body = Sentence(5..10).fake();

        let builder = Builder::default();
        let builder_err = builder.id(id).body(body).build();

        assert_eq!(builder_err, Err(BuilderError::TitleNotSet));
    }

    #[test]
    fn body_not_set() {
        let id = Faker.fake();
        let title = Sentence(2..3).fake();

        let builder = Builder::default();
        let builder_err = builder.id(id).title(title).build();

        assert_eq!(builder_err, Err(BuilderError::BodyNotSet));
    }

    #[test]
    fn id_equals_zero() {
        let title = Sentence(2..3).fake();
        let body = Sentence(5..10).fake();

        let builder = Builder::default();
        let builder_err = builder.id(0).title(title).body(body).build();

        assert_eq!(builder_err, Err(BuilderError::IdLessOrEqualsZero));
    }

    #[test]
    fn empty_title() {
        let id = Faker.fake();
        let body = Sentence(5..10).fake();

        let builder = Builder::default();
        let builder_err = builder.id(id).title(String::new()).body(body).build();

        assert_eq!(builder_err, Err(BuilderError::TitleIsEmpty));
    }

    #[test]
    fn empty_body() {
        let id = Faker.fake();
        let title = Sentence(2..3).fake();

        let builder = Builder::default();
        let builder_err = builder.id(id).title(title).body(String::new()).build();

        assert_eq!(builder_err, Err(BuilderError::BodyIsEmpty));
    }

    #[test]
    fn long_title() {
        let id = Faker.fake();
        let title =
            ((Builder::MAX_TITLE_LENGTH + 1)..(Builder::MAX_TITLE_LENGTH * 2)).fake::<String>();
        let title_length = title.len();
        let body = Sentence(5..10).fake();

        let builder = Builder::default();
        let builder_err = builder.id(id).title(title).body(body).build();

        assert_eq!(
            builder_err,
            Err(BuilderError::TitleTooLong {
                length: title_length
            })
        );
    }

    #[test]
    fn long_body() {
        let id = Faker.fake();
        let title = Sentence(2..3).fake();
        let body =
            ((Builder::MAX_BODY_LENGTH + 1)..(Builder::MAX_BODY_LENGTH * 2)).fake::<String>();
        let body_length = body.len();

        let builder = Builder::default();
        let builder_err = builder.id(id).title(title).body(body).build();

        assert_eq!(
            builder_err,
            Err(BuilderError::BodyTooLong {
                length: body_length
            })
        );
    }

    #[test]
    fn all_good() {
        let id = Faker.fake();
        let title = Sentence(2..3).fake();
        let body = Sentence(5..10).fake();

        let builder = Builder::default();

        builder.id(id).title(title).body(body).build().unwrap();
    }
}
