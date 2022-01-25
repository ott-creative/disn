use crate::config::env::API_VERSION;
use crate::response::{ApiSuccess, Success};
use axum::Json;

use crate::{error::ApiResult, service::did::DidService};

/// Generate DID JWK for user, return pub key
/// TODO: user auth
pub async fn did_create() -> ApiResult<Json<ApiSuccess<String>>> {
    let did = DidService::did_create().await?;
    Ok(Json(ApiSuccess {
        api_version: API_VERSION.to_string(),
        body: Success { data: did },
    }))
}
