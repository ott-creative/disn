#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

use futures::Future;
use sqlx::PgPool;

use poem::{listener::TcpListener, middleware::Cors, Endpoint, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;

pub mod api;
pub mod configuration;
mod dto;
mod error;
//mod extractors;
pub mod credentials;
pub mod handlers;
pub mod model;
mod response;
pub mod service;
mod sql;
pub mod telemetry;
pub mod utils;

pub mod constants;

use crate::configuration::get_configuration;
use crate::configuration::Settings;
use crate::service::chain::ChainService;
use lazy_static::lazy_static;
use sqlx::postgres::PgPoolOptions;

lazy_static! {
    pub static ref CONFIG: Settings = get_configuration().expect("Failed to read configuration.");
    pub static ref PG_POOL: PgPool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(CONFIG.database.with_db());
    pub static ref CHAIN: ChainService = ChainService::run_confirm_server(CONFIG.chain.clone());
}

fn app() -> impl Endpoint {
    let api_service = OpenApiService::new(api::DidApi, "DID Api", "1.0.0").server("/api/v1");
    let ui = api_service.swagger_ui();
    let spec = api_service.spec();
    Route::new()
        .nest("/health_check", handlers::health_check)
        .nest("/api/v1", api_service)
        .nest("/swagger", ui)
        //.nest("/passbase", post(handlers::passbase::passbase_hook))
        .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
        .with(Cors::new())
}

/// Provide database connection, and TCP listener, this can be different in production build and test build
pub fn server(
    listener: TcpListener<String>,
) -> impl Future<Output = std::result::Result<(), std::io::Error>> {
    Server::new(listener).run(app())
}
