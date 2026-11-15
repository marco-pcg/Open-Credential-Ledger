use ocl_api::{AppState, app};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    let state = AppState {
        db,
        rpc_url: std::env::var("RPC_URL").ok(),
        registry: std::env::var("REGISTRY_ADDRESS")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.parse())
            .transpose()?,
        chain_id: std::env::var("CHAIN_ID")
            .ok()
            .map(|s| s.parse())
            .transpose()?,
    };
    let listener = tokio::net::TcpListener::bind(
        std::env::var("BIND_ADDRESS").unwrap_or("127.0.0.1:8080".into()),
    )
    .await?;
    axum::serve(listener, app(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
