use std::sync::Arc;

use crate::facts::dao::Dao;

#[derive(Clone)]
pub struct AppState {
    pub dao: Arc<dyn Dao>,
}
