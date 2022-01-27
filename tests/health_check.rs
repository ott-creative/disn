use disn::configuration::get_configuration;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

#[tokio::test]
async fn health_check_works() {
    //dotenv::dotenv().ok();

    let pg_pool = prepare_db().await;
    // Arrange
    let endpoint = spawn_app(pg_pool);
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("{}/api/v1/health_check", endpoint))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch our application in the background ~somehow~
fn spawn_app(pg_pool: PgPool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = disn::server(pg_pool, listener);
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

async fn prepare_db() -> PgPool {
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_string = configuration.database.connection_string_without_db();

    let mut connection = PgConnection::connect(&connection_string)
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
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
