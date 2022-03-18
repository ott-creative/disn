use crate::service::did::DidService;
use poem_openapi::{payload::Json, ApiResponse, OpenApi};

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
    async fn create(&self) -> CreateDidResponse {
        let result = DidService::did_create().await;

        match result {
            Ok(did) => CreateDidResponse::Ok(Json(did)),
            _ => CreateDidResponse::CreateFail,
        }
    }
}
