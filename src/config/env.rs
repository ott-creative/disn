use std::{env, net::IpAddr};

use clap::Parser;

lazy_static! {
    pub static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap_or("abc".to_string());
    pub static ref API_VERSION: String = env::var("API_VERSION").unwrap_or("v1".to_string());
}

#[derive(Debug, Parser)]
pub struct ServerConfig {
    #[clap(default_value = "127.0.0.1", env)]
    pub host: IpAddr,
    #[clap(default_value = "3000", env)]
    pub port: u16,
}

#[derive(Debug, Parser)]
pub struct PgConfig {
    #[clap(default_value = "disn", env)]
    pub pg_database: String,
    #[clap(default_value = "127.0.0.1", env)]
    pub pg_host: IpAddr,
    #[clap(default_value = "5432", env)]
    pub pg_port: u16,
    #[clap(default_value = "postgres", env)]
    pub pg_user: String,
    #[clap(default_value = "", env)]
    pub pg_password: String,
}
