#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;

use crate::facts::repository::Fact;

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq, Eq))]
pub struct HttpEntity {
    id: i32,
    title: String,
    body: String,
}

#[cfg(test)]
impl HttpEntity {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

impl From<Fact> for HttpEntity {
    fn from(value: Fact) -> Self {
        HttpEntity {
            id: value.id().into(),
            title: value.title().to_owned().into(),
            body: value.body().to_owned().into(),
        }
    }
}
