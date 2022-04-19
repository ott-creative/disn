use disn::configuration::get_configuration;
use disn::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use poem::listener::TcpListener;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use disn::service::chain;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

// Launch our application in the background ~somehow~
pub async fn spawn_app() -> String {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let pg_pool = prepare_db().await;

    let listener = TcpListener::bind("127.0.0.1:3000".to_string());
    let chain = chain::ChainService::run_confirm_server(pg_pool.clone()).await;
    //let port = listener.local_addr().unwrap().port();
    let server = disn::server(pg_pool, chain, listener);
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    //let _ = tokio::spawn(server);
    println!("try to spawn server");
    tokio::spawn(async move {
        println!("spawn server");
        let _ = server.await;
    });
    //server.await;
    format!("http://127.0.0.1:{}", 3000)
}

async fn prepare_db() -> PgPool {
    let mut configuration = get_configuration().unwrap();
    configuration.database.database_name = Uuid::new_v4().to_string();
    println!("test db: {}", configuration.database.database_name);

    let mut connection = PgConnection::connect_with(&configuration.database.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(
            format!(
                r#"CREATE DATABASE "{}";"#,
                configuration.database.database_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(configuration.database.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
