use crate::{
    configuration::get_configuration,
    error::{Error, Result},
    model::{CreatePassbaseIdentity, PassbaseIdentity},
    service::{
        did::DidService,
        vc::{Credential, CredentialAdultProve, CredentialService, Credentials},
    },
};
use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    datapoints: DataPoint,
    #[serde(rename = "type")]
    data_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataPoint {
    date_of_birth: Option<String>,
    document_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub id: String,
    pub status: String,
    pub owner: Owner,
    pub score: f64,
    pub created: i64,
    pub updated: i64,
    pub resources: Vec<Resource>,
}
pub struct PassbaseService;

impl PassbaseService {
    fn process_identity_is_adult(identity_res: &IdentityResponse) -> Option<bool> {
        // check user adult result
        if identity_res.status.eq("approved")
            && identity_res.resources.len() > 0
            && identity_res.resources[0].data_type.eq("NATIONAL_ID_CARD")
        // TODO passport type
        {
            let data_point = &identity_res.resources[0].datapoints;
            // check identity date of birth
            if let Some(birth) = &data_point.date_of_birth {
                match NaiveDate::parse_from_str(birth, "%Y-%m-%d") {
                    Ok(birth) => {
                        let now = Utc::now().naive_utc();
                        let age = now.year() - birth.year();
                        tracing::info!("passbase identity age {}", age);
                        // prepare for adult vc
                        if age >= 18 {
                            return Some(true);
                        } else {
                            return Some(false);
                        }
                    }
                    Err(e) => {
                        tracing::error!("passbase identity date of birth parse error {}", e);
                    }
                }
            }
        }

        None
    }

    pub async fn refresh_identity_status(pool: &PgPool, uid: &str) -> Result<()> {
        let configuration = get_configuration().expect("Failed to read configuration.");

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://api.passbase.com/verification/v1/identities/{}",
                uid
            ))
            .header("x-api-key", configuration.passbase.secret_api_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("get passbase identity {} error {}", uid, e);
                Error::PassbaseIdentityReadError
            })?;

        let mut identity_res = response.json::<IdentityResponse>().await.map_err(|e| {
            tracing::error!("passbase identity decode response fail {} {}", uid, e);
            Error::PassbaseIdentityReadError
        })?;

        // TODO remove this test code
        // identity_res.resources[0].datapoints.date_of_birth = Some("1992-12-09".to_string());

        let adult_result = PassbaseService::process_identity_is_adult(&identity_res);

        // check if we have this identity in db
        let mut identity_db: PassbaseIdentity;
        match PassbaseIdentity::find_by_id(uid, pool).await {
            Ok(identity) => {
                identity_db = identity;
            }
            Err(e) => {
                // not exist ?
                tracing::error!("passbase identity reading error: {:?}", e);
                // TODO: check error type SqlxError(RowNotFound)
                let did = DidService::did_create(pool).await?;
                tracing::info!("created did: {}", did);
                let data = CreatePassbaseIdentity {
                    id: uid.to_string(),
                    did: Some(did),
                    identity: identity_res.owner.email,
                    status: identity_res.status,
                    is_adult: adult_result,
                    tx_hash: None,
                    is_backend_notifed: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                identity_db = PassbaseIdentity::create(data, pool).await?;
            }
        }

        // check if we can issue a vc
        match adult_result {
            Some(is_adult) => {
                let credential_signed = CredentialService::vc_credential_issue_predefined(
                    pool,
                    "ott",
                    Credential {
                        issuer_did: " ".to_string(),
                        holder_did: identity_db.did.clone().unwrap(),
                        credential: Credentials::AdultProve(CredentialAdultProve {
                            identity: identity_db.identity.clone(),
                            is_adult,
                        }),
                    },
                )
                .await?;

                tracing::info!("passbase credential signed {}", credential_signed);
                identity_db.tx_hash = Some(
                    PassbaseService::chain_submit(&identity_db.identity, credential_signed).await?,
                );

                // notify backend
                identity_db.is_backend_notifed =
                    Some(PassbaseService::notify_backend(&identity_db).await?);

                // update db
                identity_db.updated_at = Utc::now();
                PassbaseIdentity::update(identity_db, pool).await?;
            }
            None => {
                tracing::info!("passbase identity can not judge if adult {}", uid);
            }
        }

        Ok(())
    }

    async fn chain_submit(_identity: &str, _signed_credential: String) -> Result<String> {
        // TODO: chain operation
        Ok("not ready".to_string())
    }

    async fn notify_backend(_passbase_identity: &PassbaseIdentity) -> Result<bool> {
        // TODO: notify backend
        Ok(false)
    }
}
