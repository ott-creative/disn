use crate::configuration::get_configuration;
use crate::service::{
    chain::ChainService,
    did::DidService,
    vc::{Credential, CredentialService, Credentials},
};
use poem::{web::Data, Request};
use poem_openapi::{
    auth::ApiKey, param::Path, payload::Json, ApiResponse, Enum, Object, OpenApi, SecurityScheme,
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

#[derive(ApiResponse)]
enum VpPresentationResponse {
    #[oai(status = 200)]
    Ok(Json<String>),
    #[oai(status = 400)]
    PresentationFail(Json<String>),
}

#[derive(ApiResponse)]
enum VpVerifyResponse {
    #[oai(status = 200)]
    Ok(Json<String>),
    #[oai(status = 400)]
    VerifyFail(Json<String>),
}

#[derive(Enum, Debug)]
enum VcIssuerOperate {
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
struct VcIssuerCreateData {
    did: String,
    name: String,
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

#[derive(Object)]
struct VpPresentationData {
    holder_did: String,
    credential: String,
}

#[derive(Object)]
struct VpVerifyData {
    presentation: String,
}

#[derive(Object, Serialize, Deserialize)]
pub struct VcIssuePersonalIdentityData {
    pub issuer_did: String,
    pub holder_did: String,
    pub result: PartyResult,
}

#[derive(Object, Serialize, Deserialize)]
pub struct Face {
    #[serde(rename = "isIdentical")]
    pub is_identical: bool,
    pub confidence: String,
}

#[derive(Object, Serialize, Deserialize)]
pub struct PartyResult {
    pub result: PersonalInfo,
    pub face: Face,
    pub verification: PersonalVerification,
}

#[derive(Object, Serialize, Deserialize)]
pub struct PersonalInfo {
    #[serde(rename = "documentNumber")]
    pub document_number: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub sex: String,
    pub dob: String,
    pub expiry: String,
    #[serde(rename = "daysToExpiry")]
    pub days_to_expiry: i64,
    pub issued: String,
    #[serde(rename = "daysFromIssue")]
    pub days_from_issue: i64,
    pub address1: String,
    #[serde(rename = "optionalData")]
    pub optional_data: String,
    #[serde(rename = "documentType")]
    pub document_type: String,
    #[serde(rename = "documentSide")]
    pub document_side: String,
    #[serde(rename = "issueAuthority")]
    pub issue_authority: String,
    #[serde(rename = "issuerOrg_full")]
    pub issuer_org_full: String,
    #[serde(rename = "issuerOrg_iso2")]
    pub issuer_org_iso2: String,
    #[serde(rename = "issuerOrg_iso3")]
    pub issuer_org_iso3: String,
    pub nationality_full: String,
    pub nationality_iso2: String,
    pub nationality_iso3: String,
}

#[derive(Object, Serialize, Deserialize)]
pub struct PersonalVerification {
    pub passed: bool,
    pub result: PersonalVerificationResult,
}

#[derive(Object, Serialize, Deserialize)]
pub struct PersonalVerificationResult {
    pub face: bool,
    pub checkdigit: bool,
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

    /// Create VC issuer
    #[oai(path = "/vc/issuer/create", method = "post")]
    async fn vc_issuer_create(
        &self,
        data: Json<VcIssuerCreateData>,
        pool: Data<&PgPool>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerOperateResponse {
        let did = format!("did:key:{}", data.0.did);
        match CredentialService::vc_issuer_create(pool.0, &did, &data.0.name).await {
            Ok(()) => VcIssuerOperateResponse::Ok(Json("OK".to_string())),
            Err(err) => VcIssuerOperateResponse::OperateFail(Json(err.to_string())),
        }
    }

    /// Operate VC issuer Run/Restart/Disable
    #[oai(path = "/vc/issuer/operate", method = "post")]
    async fn vc_issuer_operate(
        &self,
        data: Json<VcIssuerOperateData>,
        pool: Data<&PgPool>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerOperateResponse {
        let did = format!("did:key:{}", data.0.did);
        match data.0.operation {
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
        }
    }
    /*
        #[oai(path = "/vc/issue/adult_prove", method = "post")]
        async fn vc_issuer_credential_issue(
            &self,
            pool: Data<&PgPool>,
            data: Json<VcIssueAdultProveData>,
            _auth: MyApiKeyAuthorization,
        ) -> VcIssuerIssueResponse {
            match CredentialService::vc_credential_issue_with_lib(
                pool.0,
                Credential {
                    holder_did: format!("did:key:{}", data.0.holder_did),
                    issuer_did: format!("did:key:{}", data.0.issuer_did),

                    credential: Credentials::AdultProve(CredentialAdultProve {
                        identity: data.0.identity,
                        is_adult: data.0.is_adult,
                    }),
                },
            )
            .await
            {
                Ok(signed) => VcIssuerIssueResponse::Ok(Json(signed.signed_credential)),
                Err(err) => VcIssuerIssueResponse::IssueFail(Json(err.to_string())),
            }
        }
    */
    #[oai(path = "/vc/issue/personal-identity", method = "post")]
    async fn vc_issuer_issue_personal_identity(
        &self,
        pool: Data<&PgPool>,
        chain: Data<&ChainService>,
        data: Json<VcIssuePersonalIdentityData>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerIssueResponse {
        match CredentialService::vc_credential_issue_with_lib(
            pool.0,
            chain.0,
            Credential {
                holder_did: format!("did:key:{}", data.0.holder_did),
                issuer_did: format!("did:key:{}", data.0.issuer_did),

                credential: Credentials::PersonalIdentity(data.0.result.into()),
            },
        )
        .await
        {
            Ok(signed) => VcIssuerIssueResponse::Ok(Json(signed.signed_credential)),
            Err(err) => VcIssuerIssueResponse::IssueFail(Json(err.to_string())),
        }
    }

    #[oai(path = "/vc/verify", method = "post")]
    async fn vc_issuer_credential_verify(
        &self,
        pool: Data<&PgPool>,
        chain: Data<&ChainService>,
        data: Json<VcIssueVerifyData>,
        _auth: MyApiKeyAuthorization,
    ) -> VcIssuerVerifyResponse {
        match CredentialService::vc_credential_verify_with_lib(
            pool.0,
            chain.0,
            &format!("did:key:{}", data.0.issuer_did),
            data.0.credential,
        )
        .await
        {
            Ok(_signed) => VcIssuerVerifyResponse::Ok(Json("OK".to_string())),
            Err(err) => VcIssuerVerifyResponse::VerifyFail(Json(err.to_string())),
        }
    }

    #[oai(path = "/vp/presentation", method = "post")]
    async fn vp_presentation(
        &self,
        pool: Data<&PgPool>,
        data: Json<VpPresentationData>,
        _auth: MyApiKeyAuthorization,
    ) -> VpPresentationResponse {
        match CredentialService::vp_presentation(
            pool.0,
            &format!("did:key:{}", data.0.holder_did),
            &data.0.credential,
        )
        .await
        {
            Ok(signed) => VpPresentationResponse::Ok(Json(signed)),
            Err(err) => VpPresentationResponse::PresentationFail(Json(err.to_string())),
        }
    }

    #[oai(path = "/vp/verify", method = "post")]
    async fn vp_verify(
        &self,
        pool: Data<&PgPool>,
        data: Json<VpVerifyData>,
        _auth: MyApiKeyAuthorization,
    ) -> VpVerifyResponse {
        match CredentialService::vp_verify(pool.0, data.0.presentation).await {
            Ok(_signed) => VpVerifyResponse::Ok(Json("OK".to_string())),
            Err(err) => VpVerifyResponse::VerifyFail(Json(err.to_string())),
        }
    }
}
