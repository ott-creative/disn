use crate::configuration::get_configuration;
use crate::service::{
    did::DidService,
    vc::{CredentialAdultProve, CredentialService, Credentials},
};
use poem::{web::Data, Request};
use poem_openapi::{
    auth::ApiKey, param::Path, param::Query, payload::Json, ApiResponse, Enum, Object, OpenApi,
    SecurityScheme,
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
enum DidCreateResponse {
    /// Returns when the did is successfully created.
    #[oai(status = 200)]
    Ok(Json<String>),
    /// Return when the did create fail.
    #[oai(status = 400)]
    CreateFail,
}

#[derive(ApiResponse)]
enum DidResolveResponse {
    /// Returns when the did is successfully resolved.
    #[oai(status = 200)]
    Ok(Json<String>),
    /// Return when the did create fail.
    #[oai(status = 400)]
    ResolveFail,
}

#[derive(ApiResponse)]
enum VcIssuerStatusResponse {
    /// Returns when the vc query successfully.
    #[oai(status = 200)]
    Ok(Json<String>),
    /// Return when the did create fail.
    #[oai(status = 400)]
    QueryFail,
}

#[derive(ApiResponse)]
enum VcIssuerOperateResponse {
    /// Returns when the vc operate successfully.
    #[oai(status = 200)]
    Ok(Json<String>),
    /// Return when the did create fail.
    #[oai(status = 400)]
    OperateFail(Json<String>),
}

#[derive(ApiResponse)]
enum VcIssuerIssueResponse {
    #[oai(status = 200)]
    Ok(Json<String>),
    #[oai(status = 400)]
    IssueFail(Json<String>),
}

#[derive(ApiResponse)]
enum VcIssuerVerifyResponse {
    #[oai(status = 200)]
    Ok(Json<String>),
    #[oai(status = 400)]
    VerifyFail(Json<String>),
}

#[derive(Enum, Debug)]
enum VcIssuerOperate {
    Create,
    Run,
    Restart,
    Disable,
}

#[derive(Object)]
struct VcIssuerOperateData {
    did: String,
    operation: VcIssuerOperate,
}

#[derive(Object)]
struct VcIssueAdultProveData {
    issuer_did: String,
    holder_did: String,
    identity: String,
    is_adult: bool,
}

#[derive(Object)]
struct VcIssueVerifyData {
    issuer_did: String,
    credential: String,
}

#[OpenApi]
impl DidApi {
    /// Create did
    #[oai(path = "/did/create", method = "post")]
    async fn did_create(
        &self,
        pool: Data<&PgPool>,
        _auth: MyApiKeyAuthorization,
    ) -> DidCreateResponse {
        match DidService::did_create(pool.0).await {
            Ok(did) => DidCreateResponse::Ok(Json(did)),
            _ => DidCreateResponse::CreateFail,
        }
    }

    /// Resolve did to did document
    #[oai(path = "/did/resolve/:id", method = "get")]
    async fn did_resolve(
        &self,
        _pool: Data<&PgPool>,
        id: Path<String>,
        _auth: MyApiKeyAuthorization,
    ) -> DidResolveResponse {
        match DidService::did_resolve(&format!("did:key:{}", id.0)).await {
            Ok(doc) => DidResolveResponse::Ok(Json(doc)),
            _ => DidResolveResponse::ResolveFail,
        }
    }

    /// Get VC Issuer status
    #[oai(path = "/vc/issuer/:did/status", method = "get")]
    async fn vc_issuer_status(
        &self,
        pool: Data<&PgPool>,
        did: Path<String>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerStatusResponse {
        match CredentialService::vc_issuer_get_by_did(pool.0, &format!("did:key:{}", did.0)).await {
            Ok(vc_issuer) => VcIssuerStatusResponse::Ok(Json(vc_issuer.status.to_string())),
            _ => VcIssuerStatusResponse::QueryFail,
        }
    }

    /// Operate VC issuer create/run/disable
    #[oai(path = "/vc/issuer", method = "post")]
    async fn vc_issuer_operate(
        &self,
        data: Json<VcIssuerOperateData>,
        pool: Data<&PgPool>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerOperateResponse {
        let did = format!("did:key:{}", data.0.did);
        match data.0.operation {
            VcIssuerOperate::Create => {
                match CredentialService::vc_issuer_create(pool.0, &did).await {
                    Ok(()) => VcIssuerOperateResponse::Ok(Json("OK".to_string())),
                    Err(err) => VcIssuerOperateResponse::OperateFail(Json(err.to_string())),
                }
            }
            VcIssuerOperate::Run => {
                match CredentialService::vc_issuer_service_run(pool.0, &did, false).await {
                    Ok(()) => VcIssuerOperateResponse::Ok(Json("OK".to_string())),
                    Err(err) => VcIssuerOperateResponse::OperateFail(Json(err.to_string())),
                }
            }
            VcIssuerOperate::Restart => {
                match CredentialService::vc_issuer_service_run(pool.0, &did, true).await {
                    Ok(()) => VcIssuerOperateResponse::Ok(Json("OK".to_string())),
                    Err(err) => VcIssuerOperateResponse::OperateFail(Json(err.to_string())),
                }
            }
            VcIssuerOperate::Disable => {
                match CredentialService::vc_issuer_service_disable(pool.0, &did).await {
                    Ok(()) => VcIssuerOperateResponse::Ok(Json("OK".to_string())),
                    Err(err) => VcIssuerOperateResponse::OperateFail(Json(err.to_string())),
                }
            }
            _ => VcIssuerOperateResponse::OperateFail(Json("Unknown error".to_string())),
        }
    }

    #[oai(path = "/vc/issue/adult_prove", method = "post")]
    async fn vc_issuer_credential_issue(
        &self,
        pool: Data<&PgPool>,
        data: Json<VcIssueAdultProveData>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerIssueResponse {
        match CredentialService::vc_credential_issue(
            pool.0,
            &format!("did:key:{}", data.0.issuer_did),
            Credentials::AdultProve(CredentialAdultProve {
                identity: data.0.identity,
                holder_did: format!("did:key:{}", data.0.holder_did),
                issuer_did: format!("did:key:{}", data.0.issuer_did),
                is_adult: data.0.is_adult,
            }),
        )
        .await
        {
            Ok(signed) => VcIssuerIssueResponse::Ok(Json(signed)),
            Err(err) => VcIssuerIssueResponse::IssueFail(Json(err.to_string())),
        }
    }

    #[oai(path = "/vc/verify", method = "post")]
    async fn vc_issuer_credential_verify(
        &self,
        pool: Data<&PgPool>,
        data: Json<VcIssueVerifyData>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerVerifyResponse {
        match CredentialService::vc_credential_verify(
            pool.0,
            &format!("did:key:{}", data.0.issuer_did),
            data.0.credential,
        )
        .await
        {
            Ok(_signed) => VcIssuerVerifyResponse::Ok(Json("OK".to_string())),
            Err(err) => VcIssuerVerifyResponse::VerifyFail(Json(err.to_string())),
        }
    }
}
