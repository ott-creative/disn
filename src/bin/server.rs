use disn::configuration::get_configuration;
use disn::service::chain;
use disn::service::vc;
use disn::telemetry::{get_subscriber, init_subscriber};
use poem::listener::TcpListener;
use sqlx::postgres::PgPoolOptions;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    //let subscriber = get_subscriber("disn".into(), "info".into(), std::io::stdout);
    //init_subscriber(subscriber);

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let configuration = get_configuration().expect("Failed to read configuration.");

    let pg_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // disable
    // let _ = vc::CredentialService::vc_issuer_service_restart(&pg_pool).await;
    let _ = vc::CredentialService::load_predefined_vc_issuers(&pg_pool).await;
    let chain = chain::ChainService::run_confirm_server(pg_pool.clone()).await;
    let addr = format!(
        "{}:{}",
        configuration.server.host, configuration.server.port
    );
    tracing::info!("listening on {}", addr);
    let listener = TcpListener::bind(addr);

    let server = disn::server(pg_pool, chain, listener);

    if let Err(err) = server.await {
        tracing::error!("server error : {:?}", err);
    }
}
