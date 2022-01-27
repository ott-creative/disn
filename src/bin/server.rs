use disn::configuration::get_configuration;
use std::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let configuration = get_configuration().expect("Failed to read configuration.");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    let pg_pool = sqlx::PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let addr = format!(
        "{}:{}",
        configuration.server.host, configuration.server.port
    );
    tracing::info!("listening on {}", addr);
    let listener = TcpListener::bind(addr).expect("Failed to bind on port");

    let server = disn::server(pg_pool, listener);

    if let Err(err) = server.await {
        tracing::error!("server error : {:?}", err);
    }
}
