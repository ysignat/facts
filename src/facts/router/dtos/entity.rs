#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;

use crate::facts::dao::Entity;

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq, Eq))]
pub struct HttpEntity {
    id: u64,
    title: String,
    body: String,
}

#[cfg(test)]
impl HttpEntity {
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

impl From<Entity> for HttpEntity {
    fn from(value: Entity) -> Self {
        HttpEntity {
            id: value.id(),
            title: value.title().to_owned(),
            body: value.body().to_owned(),
        }
    }
}
