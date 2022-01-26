use axum::async_trait;
use clap::Parser;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::{db::DbPool, env::PgConfig};

#[async_trait]
impl DbPool for PgPool {
    async fn retrieve(has_db: bool) -> Self {
        let config = PgConfig::parse();
        let uri = if has_db {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                config.pg_user,
                config.pg_password,
                config.pg_host,
                config.pg_port,
                config.pg_database
            )
        } else {
            format!(
                "postgres://{}:{}@{}:{}",
                config.pg_user, config.pg_password, config.pg_host, config.pg_port
            )
        };

        PgPoolOptions::new()
            .connect(&uri)
            .await
            .expect("DB connection was failed")
    }
}
