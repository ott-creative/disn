use crate::service::did::DidService;
use poem::{web::Data, Endpoint};
use poem_openapi::{param::Path, payload::Json, ApiResponse, OpenApi};
use sqlx::PgPool;

pub struct DidApi;

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
    async fn create(&self, pool: Data<&PgPool>) -> CreateDidResponse {
        match DidService::did_create(&pool.0).await {
            Ok(did) => CreateDidResponse::Ok(Json(did)),
            _ => CreateDidResponse::CreateFail,
        }
    }

    #[oai(path = "/did/resolve/:id", method = "get")]
    async fn resolve(&self, _pool: Data<&PgPool>, id: Path<String>) -> CreateDidResponse {
        match DidService::did_resolve(&id.0).await {
            Ok(doc) => CreateDidResponse::Ok(Json(doc)),
            _ => CreateDidResponse::CreateFail,
        }
    }
}
