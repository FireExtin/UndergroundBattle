use hegemony_server::service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let database = std::env::var("HEGEMONY_DB").unwrap_or_else(|_| "rust-game-v2.2.sqlite3".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8090".into());
    let address = format!("0.0.0.0:{port}");
    let app = service::router(service::Store::open(&database)?);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(%address, %database, "authoritative Hegemony server ready");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
