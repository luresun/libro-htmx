use dotenvy::dotenv;
use tracing::info;

use libro_htmx::infra::{app::create_app, setup::init_app_state};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let app_state = init_app_state().await?;

    let app = create_app(app_state);
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    // let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
       // .await
       // .unwrap();

    info!("Backend listening at {}", &listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
