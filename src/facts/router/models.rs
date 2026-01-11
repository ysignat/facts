use serde::{Deserialize, Serialize};

use crate::facts::repository::{
    CreateFactRequest,
    CreateFactRequestError,
    Fact,
    FactBody,
    FactTitle,
};

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq, Eq))]
pub struct HttpFactResponse {
    id: i32,
    title: String,
    body: String,
}

#[cfg(test)]
impl HttpFactResponse {
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

impl From<Fact> for HttpFactResponse {
    fn from(value: Fact) -> Self {
        HttpFactResponse {
            id: value.id().into(),
            title: value.title().to_owned().into(),
            body: value.body().to_owned().into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize, PartialEq, Eq))]
pub struct HttpCreateFactRequestBody {
    title: String,
    body: String,
}

#[cfg(test)]
impl HttpCreateFactRequestBody {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

impl TryFrom<HttpCreateFactRequestBody> for CreateFactRequest {
    type Error = CreateFactRequestError;

    fn try_from(value: HttpCreateFactRequestBody) -> Result<Self, Self::Error> {
        Ok(CreateFactRequest::new(
            &FactTitle::new(&value.title)?,
            &FactBody::new(&value.body)?,
        ))
    }
}
