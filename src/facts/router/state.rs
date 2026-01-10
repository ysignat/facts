use std::sync::Arc;

use crate::facts::repository::FactsRepository;

#[derive(Clone)]
pub struct AppState {
    pub facts: Arc<dyn FactsRepository>,
}
