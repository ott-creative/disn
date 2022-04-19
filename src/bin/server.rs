use disn::service::vc;
use poem::listener::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use disn::PG_POOL;
use disn::CONFIG;
use disn::CHAIN;

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

    // disable
    // let _ = vc::CredentialService::vc_issuer_service_restart(&pg_pool).await;
    let _ = vc::CredentialService::load_predefined_vc_issuers(&PG_POOL).await;
    let addr = format!(
        "{}:{}",
        CONFIG.server.host, CONFIG.server.port
    );
    tracing::info!("listening on {}", addr);
    let listener = TcpListener::bind(addr);

    let server = disn::server(listener);

    if let Err(err) = server.await {
        tracing::error!("server error : {:?}", err);
    }
}
