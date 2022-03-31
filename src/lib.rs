#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

use futures::Future;
use sqlx::PgPool;

use poem::{listener::TcpListener, middleware::Cors, Endpoint, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;

mod api;
pub mod configuration;
mod dto;
mod error;
//mod extractors;
mod handlers;
mod model;
mod response;
pub mod service;
mod sql;
pub mod telemetry;
mod utils;

pub mod constants;

fn app(pg_pool: PgPool) -> impl Endpoint {
    let api_service = OpenApiService::new(api::DidApi, "DID Api", "1.0.0").server("/");
    let ui = api_service.swagger_ui();
    let spec = api_service.spec();
    Route::new()
        .nest("/health_check", handlers::health_check)
        .nest("/", api_service)
        .nest("/ui", ui)
        .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
        .with(Cors::new())
        .data(pg_pool)
}

/// Provide database connection, and TCP listener, this can be different in production build and test build
pub fn server(
    pg_pool: PgPool,
    listener: TcpListener<String>,
) -> impl Future<Output = std::result::Result<(), std::io::Error>> {
    Server::new(listener).run(app(pg_pool))
}
