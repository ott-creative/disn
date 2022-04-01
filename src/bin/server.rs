use disn::configuration::get_configuration;
use disn::service::vc;
use disn::telemetry::{get_subscriber, init_subscriber};
use poem::listener::TcpListener;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let subscriber = get_subscriber("disn".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let pg_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // TODO: check restart tasks
    let _ = vc::CredentialService::vc_issuer_service_restart(&pg_pool).await;
    let _ = vc::CredentialService::load_predefined_vc_issuers(&pg_pool).await;

    let addr = format!(
        "{}:{}",
        configuration.server.host, configuration.server.port
    );
    tracing::info!("listening on {}", addr);
    let listener = TcpListener::bind(addr);

    let server = disn::server(pg_pool, listener);

    if let Err(err) = server.await {
        tracing::error!("server error : {:?}", err);
    }
}
