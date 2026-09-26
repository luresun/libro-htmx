use crate::{
    adapters::{crypto::argon2::ArgonPasswordHasher, persistence::PostgresPersistence},
    infra::db::init_db,
};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::{info, warn};

pub mod app;
pub mod config;
pub mod db;
pub mod setup;

pub async fn postgres_persistence() -> anyhow::Result<PostgresPersistence> {
    let pool = init_db().await?;
    let persistence = PostgresPersistence::new(pool);
    Ok(persistence)
}

pub async fn session_store() -> anyhow::Result<PostgresStore> {
    let pool = init_db().await?;
    let store = PostgresStore::new(pool);
    info!("Session store created, migrating...");
    match store.migrate().await {
        Ok(_) => info!("Session store migration successful"),
        Err(e) => {
            warn!("Session store migration failed: {:?}", e);
            // Continue anyway - table might already exist
        }
    }
    Ok(store)
}

pub fn argon2_password_hasher() -> ArgonPasswordHasher {
    ArgonPasswordHasher::default()
}
