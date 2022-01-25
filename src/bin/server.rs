use clap::Parser;
use disn::config;
use std::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    use config::db::DbPool;
    let pg_pool = sqlx::PgPool::retrieve().await;
    let config = config::env::ServerConfig::parse();
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("listening on {}", addr);
    let listener = TcpListener::bind(addr).expect("Failed to bind on port");

    let server = disn::server(pg_pool, listener);

    if let Err(err) = server.await {
        tracing::error!("server error : {:?}", err);
    }
}
