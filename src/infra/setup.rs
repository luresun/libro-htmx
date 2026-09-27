use crate::{
    adapters::http::app_state::AppState,
    infra::{argon2_password_hasher, config::AppConfig, postgres_persistence, session_store},
    use_cases::user::UserUseCases,
    use_cases::post::PostUseCases,
};
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub async fn init_app_state() -> anyhow::Result<AppState> {
    let config = AppConfig::from_env();

    let postgres_arc = Arc::new(postgres_persistence().await?);
    let session_store = session_store().await?;
    let argon_hasher = argon2_password_hasher();

    let user_use_cases = UserUseCases::new(Arc::new(argon_hasher), postgres_arc.clone());
    let post_use_cases = PostUseCases::new(postgres_arc.clone());

    Ok(AppState {
        config: Arc::new(config),
        user_use_cases: Arc::new(user_use_cases),
        post_use_cases: Arc::new(post_use_cases),
        session_store,
    })
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "axum_trainer=debug,tower_http=debug".into());

    // Console (pretty logs) - outputs to stdout/stderr
    let console_layer = fmt::layer()
        .with_target(false) // don't show target (module path)
        .with_level(true) // show log level
        .pretty(); // human-friendly, with colors

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .try_init()
        .ok();
}
