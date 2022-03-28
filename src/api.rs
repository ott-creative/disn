use crate::configuration::get_configuration;
use crate::service::did::DidService;
use poem::{web::Data, Endpoint, Request};
use poem_openapi::{
    auth::ApiKey, param::Path, payload::Json, ApiResponse, OpenApi, SecurityScheme,
};
use sqlx::PgPool;

pub struct DidApi;

/// ApiKey authorization
#[derive(SecurityScheme)]
#[oai(
    type = "api_key",
    key_name = "X-API-Key",
    in = "header",
    checker = "api_checker"
)]
struct MyApiKeyAuthorization(());

async fn api_checker(_req: &Request, api_key: ApiKey) -> Option<()> {
    //let server_key = req.data::<ServerKey>().unwrap();
    //VerifyWithKey::<()>::verify_with_key(api_key.key.as_str(), server_key).ok()
    let configuration = get_configuration().expect("Failed to read configuration.");
    if api_key.key.eq(&configuration.did.api_key) {
        Some(())
    } else {
        None
    }
}

#[derive(ApiResponse)]
enum CreateDidResponse {
    /// Returns when the user is successfully created.
    #[oai(status = 200)]
    Ok(Json<String>),
    /// Return when the did create fail.
    #[oai(status = 400)]
    CreateFail,
}

#[OpenApi]
impl DidApi {
    /// Create did
    #[oai(path = "/did/create", method = "post")]
    async fn create(&self, pool: Data<&PgPool>, _auth: MyApiKeyAuthorization) -> CreateDidResponse {
        match DidService::did_create(&pool.0).await {
            Ok(did) => CreateDidResponse::Ok(Json(did)),
            _ => CreateDidResponse::CreateFail,
        }
    }

    #[oai(path = "/did/resolve/:id", method = "get")]
    async fn resolve(
        &self,
        _pool: Data<&PgPool>,
        id: Path<String>,
        _auth: MyApiKeyAuthorization,
    ) -> CreateDidResponse {
        match DidService::did_resolve(&id.0).await {
            Ok(doc) => CreateDidResponse::Ok(Json(doc)),
            _ => CreateDidResponse::CreateFail,
        }
    }
}
