use std::fmt;

#[cfg(test)]
use fake::{faker::lorem::en::Sentence, Dummy, Fake, Faker};
use thiserror::Error;

#[derive(Clone)]
#[cfg_attr(test, derive(Dummy, Eq, PartialEq, Debug))]
pub struct Fact {
    id: FactId,
    title: FactTitle,
    body: FactBody,
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum FactError {
    #[error("Id is invalid: {inner}")]
    InvalidId { inner: String },
    #[error("Title is invalid: {inner}")]
    InvalidTitle { inner: String },
    #[error("Body is invalid: {inner}")]
    InvalidBody { inner: String },
}

impl From<FactIdError> for FactError {
    fn from(value: FactIdError) -> Self {
        Self::InvalidId {
            inner: value.to_string(),
        }
    }
}

impl From<FactTitleError> for FactError {
    fn from(value: FactTitleError) -> Self {
        Self::InvalidTitle {
            inner: value.to_string(),
        }
    }
}

impl From<FactBodyError> for FactError {
    fn from(value: FactBodyError) -> Self {
        Self::InvalidBody {
            inner: value.to_string(),
        }
    }
}

impl Fact {
    pub fn new(id: i32, title: &str, body: &str) -> Result<Self, FactError> {
        let id = FactId::new(id)?;
        let title = FactTitle::new(title)?;
        let body = FactBody::new(body)?;

        Ok(Self { id, title, body })
    }

    pub fn id(&self) -> FactId {
        self.id
    }

    pub fn title(&self) -> &FactTitle {
        &self.title
    }

    pub fn body(&self) -> &FactBody {
        &self.body
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub struct FactId(i32);

impl From<FactId> for i32 {
    fn from(val: FactId) -> Self {
        val.0
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum FactIdError {
    #[error("Id is non-positive")]
    NonPositive,
}

#[cfg(test)]
impl Dummy<Faker> for FactId {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &Faker, _: &mut R) -> Self {
        Self((1..i32::MAX).fake())
    }
}

impl FactId {
    pub fn new(raw: i32) -> Result<Self, FactIdError> {
        if raw.is_positive() {
            Ok(Self(raw))
        } else {
            Err(FactIdError::NonPositive)
        }
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub struct FactTitle(String);

impl From<FactTitle> for String {
    fn from(val: FactTitle) -> Self {
        val.0
    }
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum FactTitleError {
    #[error("Title is very long: {length:?} chars")]
    TooLong { length: usize },
    #[error("Empty title is not allowed")]
    IsEmpty,
}

#[cfg(test)]
impl Dummy<Faker> for FactTitle {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &Faker, _: &mut R) -> Self {
        let raw = Sentence(2..3).fake::<String>();
        match raw.char_indices().nth(Self::MAX_LENGTH) {
            Some((idx, _)) => Self(raw[..idx].to_string()),
            None => Self(raw),
        }
    }
}

impl FactTitle {
    const MAX_LENGTH: usize = 64;

    pub fn new(raw: &str) -> Result<Self, FactTitleError> {
        if raw.is_empty() {
            return Err(FactTitleError::IsEmpty);
        }

        if raw.len().gt(&Self::MAX_LENGTH) {
            return Err(FactTitleError::TooLong { length: raw.len() });
        }

        Ok(Self(raw.to_string()))
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub struct FactBody(String);

impl From<FactBody> for String {
    fn from(val: FactBody) -> Self {
        val.0
    }
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum FactBodyError {
    #[error("Body is very long: {length:?} chars")]
    TooLong { length: usize },
    #[error("Empty body is not allowed")]
    IsEmpty,
}

#[cfg(test)]
impl Dummy<Faker> for FactBody {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &Faker, _: &mut R) -> Self {
        let raw = Sentence(2..3).fake::<String>();
        match raw.char_indices().nth(Self::MAX_LENGTH) {
            Some((idx, _)) => Self(raw[..idx].to_string()),
            None => Self(raw),
        }
    }
}

impl FactBody {
    const MAX_LENGTH: usize = 2048;

    pub fn new(raw: &str) -> Result<Self, FactBodyError> {
        if raw.is_empty() {
            return Err(FactBodyError::IsEmpty);
        }

        if raw.len().gt(&Self::MAX_LENGTH) {
            return Err(FactBodyError::TooLong { length: raw.len() });
        }

        Ok(Self(raw.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    #[test]
    fn non_positive_id() {
        assert_eq!(
            FactId::new((i32::MIN..=0).fake()),
            Err(FactIdError::NonPositive)
        );
    }

    #[test]
    fn empty_title() {
        assert_eq!(FactTitle::new(""), Err(FactTitleError::IsEmpty));
    }

    #[test]
    fn empty_body() {
        assert_eq!(FactBody::new(""), Err(FactBodyError::IsEmpty));
    }

    #[test]
    fn long_title() {
        let title = ((FactTitle::MAX_LENGTH + 1)..(FactTitle::MAX_LENGTH * 2)).fake::<String>();

        assert_eq!(
            FactTitle::new(&title),
            Err(FactTitleError::TooLong {
                length: title.len()
            })
        );
    }

    #[test]
    fn long_body() {
        let body = ((FactBody::MAX_LENGTH + 1)..(FactBody::MAX_LENGTH * 2)).fake::<String>();

        assert_eq!(
            FactBody::new(&body),
            Err(FactBodyError::TooLong { length: body.len() })
        );
    }

    #[test]
    fn good_fact() {
        let id = Faker.fake::<FactId>().0;
        let title = Faker.fake::<FactTitle>().0;
        let body = Faker.fake::<FactBody>().0;

        Fact::new(id, &title, &body).unwrap();
    }

    #[test]
    fn fact_with_invalid_id() {
        let title = Faker.fake::<FactTitle>().0;
        let body = Faker.fake::<FactBody>().0;

        assert!(matches!(
            Fact::new(0, &title, &body),
            Err(FactError::InvalidId { inner: _ })
        ));
    }

    #[test]
    fn fact_with_invalid_title() {
        let id = Faker.fake::<FactId>().0;
        let body = Faker.fake::<FactBody>().0;

        assert!(matches!(
            Fact::new(id, "", &body),
            Err(FactError::InvalidTitle { inner: _ })
        ));
    }

    #[test]
    fn fact_with_invalid_body() {
        let id = Faker.fake::<FactId>().0;
        let title = Faker.fake::<FactTitle>().0;

        assert!(matches!(
            Fact::new(id, &title, ""),
            Err(FactError::InvalidBody { inner: _ })
        ));
    }
}
