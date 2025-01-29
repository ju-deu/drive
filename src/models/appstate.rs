use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use sqlx::{Pool, Postgres};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone)]
pub struct Appstate {
    pub(crate) db_pool: Arc<Pool<Postgres>>,
    pub(crate) jwt_secret: String,
    pub(crate) cookie_secret: Key,
    pub file_location: String,
}

/// This wrapper is used because the trait `axum_core::extract::from_ref` cannot be implemented for
/// arbitrary types in the current scope
#[derive(Clone)]
pub struct AppstateWrapper(pub Arc<Appstate>);

impl Appstate {
    pub fn new(db_pool: Arc<Pool<Postgres>>, jwt_secret: String, cookie_secret: Key, file_location: String) -> Self {
        Self {
            db_pool,
            jwt_secret,
            cookie_secret,
            file_location
        }
    }
}

impl Deref for AppstateWrapper {
    type Target = Appstate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppstateWrapper> for Key {
    fn from_ref(state: &AppstateWrapper) -> Self {
        state.0.cookie_secret.clone()
    }
}

impl FromRef<Appstate> for Key {
    fn from_ref(state: &Appstate) -> Self {
        state.cookie_secret.clone()
    }
}
