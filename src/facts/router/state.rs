use std::sync::Arc;

#[cfg(test)]
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
    PasswordHasher,
};

use crate::facts::FactsRepository;
#[cfg(test)]
use crate::facts::MockedFactsRepository;

#[derive(Clone)]
pub struct AppState {
    pub facts: Arc<dyn FactsRepository>,
    pub auth_key: String,
}

#[cfg(test)]
impl Default for AppState {
    fn default() -> Self {
        Self {
            facts: Arc::new(MockedFactsRepository {}),
            auth_key: Argon2::default()
                .hash_password(&[], &SaltString::generate(&mut OsRng))
                .unwrap()
                .to_string(),
        }
    }
}
