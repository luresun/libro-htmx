use std::sync::Arc;

use axum::extract::FromRef;
use tower_sessions_sqlx_store::PostgresStore;

use crate::{
    infra::config::AppConfig, use_cases::post::PostUseCases, use_cases::user::UserUseCases,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub user_use_cases: Arc<UserUseCases>,
    pub post_use_cases: Arc<PostUseCases>,
    pub session_store: PostgresStore,
}

impl FromRef<AppState> for Arc<UserUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.user_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<crate::infra::config::AppConfig> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.config.clone()
    }
}

impl FromRef<AppState> for Arc<PostUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.post_use_cases.clone()
    }
}

impl FromRef<AppState> for PostgresStore {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.session_store.clone()
    }
}
