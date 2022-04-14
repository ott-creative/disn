use crate::{
    configuration::get_configuration,
    error::{Error, Result},
    model::{CreateVcIssuerData, UpdateVcIssuerData, VcIssuer},
    service::did::DidService,
};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{collections::HashMap, process::Command};
use uuid::Uuid;

use did_method_key::DIDKey;
use didkit::{LinkedDataProofOptions, VerifiableCredential, JWK};

pub struct CredentialService;

pub struct CredentialAdultProve {
    pub identity: String,
    pub is_adult: bool,
}

#[derive(Deserialize)]
pub struct VerifyResult {
    pub checks: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Deserialize)]
pub struct IssueResult {
    pub issuer_did: String,
    pub holder_did: String,
    pub signed_credential: String,
}

impl CredentialAdultProve {
    pub fn generate_json(&self, issuer: &str, holder: &str) -> Value {
        json!({
            "@context": ["https://www.w3.org/2018/credentials/v1", "https://credential.codegene.xyz/context/adult.jsonld"],
            "id": format!("urn:uuid:{}", Uuid::new_v4().to_string()),
            "type": ["VerifiableCredential"],
            "issuer": issuer,
            "holder": holder,
            "credentialSubject": {
                "id": holder,
                "email": self.identity,
                "isAdult": self.is_adult,
            }
        })
    }
}

pub enum Credentials {
    AdultProve(CredentialAdultProve),
}

pub struct Credential {
    pub issuer_did: String,
    pub holder_did: String,
    pub credential: Credentials,
}

pub enum CredentialServiceStatus {
    Created,
    Running,
    Disabled,
    Error,
}

impl From<i32> for CredentialServiceStatus {
    fn from(status: i32) -> Self {
        match status {
            0 => CredentialServiceStatus::Created,
            1 => CredentialServiceStatus::Running,
            2 => CredentialServiceStatus::Disabled,
            3 => CredentialServiceStatus::Error,
            _ => CredentialServiceStatus::Error,
        }
    }
}

impl From<CredentialServiceStatus> for i32 {
    fn from(status: CredentialServiceStatus) -> Self {
        match status {
            CredentialServiceStatus::Created => 0,
            CredentialServiceStatus::Running => 1,
            CredentialServiceStatus::Disabled => 2,
            CredentialServiceStatus::Error => 3,
        }
    }
}

impl From<CredentialServiceStatus> for String {
    fn from(status: CredentialServiceStatus) -> Self {
        match status {
            CredentialServiceStatus::Created => "created".to_string(),
            CredentialServiceStatus::Running => "running".to_string(),
            CredentialServiceStatus::Disabled => "disabled".to_string(),
            CredentialServiceStatus::Error => "error".to_string(),
        }
    }
}

use crate::model::Did;

impl CredentialService {
    /// create a new vc issuer
    pub async fn vc_issuer_create(pool: &PgPool, did: &str, name: &str) -> Result<()> {
        let data = CreateVcIssuerData {
            did: did.to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            service_address: 0,
            status: CredentialServiceStatus::Created.into(),
        };

        VcIssuer::create(data, &pool).await?;

        Ok(())
    }

    /// get available service address(port)
    pub async fn vc_issuer_get_available_port(pool: &PgPool) -> Result<u16> {
        let settings = get_configuration().expect("Failed to read configuration.");
        let mut port = settings.did.vc_issuer_port_start;
        let issuers = VcIssuer::find_all(&pool).await?;
        if issuers.len() > 0 && issuers[0].service_address > 0 {
            port = (issuers[0].service_address + 1) as u16;
        }

        Ok(port)
    }

    /// get vc issuer by did
    pub async fn vc_issuer_get_by_did(pool: &PgPool, did: &str) -> Result<VcIssuer> {
        VcIssuer::find_by_did(did, pool).await
    }

    /// start vc issuer service
    pub async fn vc_issuer_service_run(pool: &PgPool, did: &str, force_start: bool) -> Result<()> {
        tracing::info!("Try to run vc issuer: {}", did);
        // read vc issuer, check if it's running
        let mut issuer = VcIssuer::find_by_did(did, &pool).await?;
        if !force_start && issuer.status == CredentialServiceStatus::Running as i32 {
            tracing::info!("Issuer {} already running", did);
            return Ok(());
        }

        // read out related private key
        let did_instance = Did::find_by_id(did, pool).await?;

        // check if can get available port
        if issuer.service_address == 0 {
            let port = CredentialService::vc_issuer_get_available_port(pool).await?;
            issuer.service_address = port as i32;
        }

        tracing::info!("assign vc issuer port: {}", issuer.service_address);

        // start didkit-http service
        let settings = get_configuration().expect("Failed to read configuration.");
        let mut run_didhttp = Command::new(format!("{}/didkit-http", settings.did.didkit_path));
        run_didhttp.arg("-p");
        run_didhttp.arg(issuer.service_address.to_string());
        run_didhttp.arg("-j");
        run_didhttp.arg(did_instance.jwk);
        //.output()?;
        match run_didhttp.spawn() {
            Ok(child) => {
                let data = UpdateVcIssuerData {
                    status: CredentialServiceStatus::Running.into(),
                    name: issuer.name,
                    updated_at: Utc::now(),
                    did: did.to_string(),
                    pid: Some(child.id() as i32),
                    service_address: issuer.service_address,
                };
                VcIssuer::update(data, &pool).await?;
                Ok(())
            }
            Err(e) => {
                tracing::error!("vc issuer {} failed to start: {}", did, e);
                Err(Error::VcIssuerRunningError)
            }
        }
    }

    /// stop vc issuer service
    pub async fn vc_issuer_service_disable(pool: &PgPool, did: &str) -> Result<()> {
        // read vc issuer, check if it's running
        let issuer = VcIssuer::find_by_did(did, &pool).await?;
        if issuer.status == CredentialServiceStatus::Disabled as i32 {
            tracing::info!("Issuer {} already stopped", did);
            return Ok(());
        }

        match issuer.pid {
            // kill didkit-http pid
            Some(pid) => {
                let mut kill_didhttp = Command::new(format!("kill"));
                kill_didhttp.arg("-9");
                kill_didhttp.arg(format!("{}", pid));
                kill_didhttp.output()?;
            }
            None => {}
        }

        // update vc issuer status
        let data = UpdateVcIssuerData {
            status: CredentialServiceStatus::Disabled.into(),
            name: issuer.name,
            updated_at: Utc::now(),
            did: did.to_string(),
            pid: None,
            service_address: issuer.service_address,
        };

        VcIssuer::update(data, &pool).await?;

        Ok(())
    }

    /// restart all vc issuers which db status is running
    pub async fn vc_issuer_service_restart(pool: &PgPool) -> Result<()> {
        let issuers = VcIssuer::find_all(&pool).await?;
        for issuer in issuers {
            if issuer.status == CredentialServiceStatus::Running as i32 {
                CredentialService::vc_issuer_service_run(&pool, &issuer.did, true).await?;
            }
        }

        Ok(())
    }

    /// issue credential
    pub async fn vc_credential_issue(pool: &PgPool, credential: Credential) -> Result<IssueResult> {
        // check if this issuer is running
        let issuer = VcIssuer::find_by_did(&credential.issuer_did, &pool).await?;
        if issuer.status != CredentialServiceStatus::Running as i32 {
            tracing::info!("Issuer {} not running", &credential.issuer_did);
            return Err(Error::VcIssuerNotRunning);
            // TODO: should we start service here?
        }

        // prepare unsigned credential
        let credential_unsigned;
        match credential.credential {
            Credentials::AdultProve(adult_prove) => {
                //credential_unsigned = adult_prove.generate_unsigned();
                credential_unsigned =
                    adult_prove.generate_json(&issuer.did, &credential.holder_did);
                tracing::info!("credential unsigned:{}", credential_unsigned);
            }
        }

        // sign credential
        let client = reqwest::Client::new();
        let body = json!({
          "credential": credential_unsigned,
          "options": {
            "verificationMethod": format!("{}#{}", issuer.did, issuer.did.chars().skip(8).collect::<String>()),
            "proofPurpose": "assertionMethod"
          }
        });

        // Act
        let response = client
            .post(format!(
                "http://127.0.0.1:{}/issue/credentials",
                issuer.service_address
            ))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    "vc issuer {} failed to issue credential: {}",
                    &credential.issuer_did,
                    e
                );
                Error::VcIssueError
            })?;

        Ok(IssueResult {
            signed_credential: response
                .text()
                .await
                .map_err(|e| {
                    tracing::error!(
                        "vc issuer {} failed to parse response: {}",
                        &credential.issuer_did,
                        e
                    );
                    Error::VcIssueError
                })?
                .to_string(),
            issuer_did: issuer.did,
            holder_did: credential.holder_did,
        })
    }

    //. issue credential with lib access
    pub async fn vc_credential_issue_with_lib(
        pool: &PgPool,
        credential: Credential,
    ) -> Result<IssueResult> {
        // check if this issuer is running
        // TODO: join query
        let issuer = VcIssuer::find_by_did(&credential.issuer_did, &pool).await?;
        let issuer_did = Did::find_by_id(&issuer.did, pool).await?;

        let credential_unsigned;
        match credential.credential {
            Credentials::AdultProve(adult_prove) => {
                //credential_unsigned = adult_prove.generate_unsigned();
                credential_unsigned =
                    adult_prove.generate_json(&issuer.did, &credential.holder_did);
                tracing::info!("credential unsigned:{}", credential_unsigned);
            }
        }

        let key: JWK = serde_json::from_str(&issuer_did.jwk)?;
        let mut verifiable_credential: VerifiableCredential =
            serde_json::from_value(credential_unsigned)?;
        let proof = verifiable_credential
            .generate_proof(&key, &LinkedDataProofOptions::default(), &DIDKey)
            .await?;

        verifiable_credential.add_proof(proof);

        let signed_credential = serde_json::to_vec(&verifiable_credential)?;

        Ok(IssueResult {
            signed_credential: String::from_utf8(signed_credential)?,
            issuer_did: issuer.did,
            holder_did: credential.holder_did,
        })
    }

    /// verify credential
    pub async fn vc_credential_verify(
        pool: &PgPool,
        issuer_did: &str,
        signed_credential: String,
    ) -> Result<bool> {
        // check if this issuer is running
        let issuer = VcIssuer::find_by_did(issuer_did, &pool).await?;
        if issuer.status != CredentialServiceStatus::Running as i32 {
            tracing::info!("Issuer {} not running", issuer_did);
            return Err(Error::VcIssuerNotRunning);
            // TODO: should we start service here?
        }

        let credential: Value = serde_json::from_str(&signed_credential).map_err(|e| {
            tracing::error!("vc issuer {} failed to parse credential: {}", issuer_did, e);
            Error::VcVerifyParserJsonError
        })?;

        let client = reqwest::Client::new();
        let body = json!({
          "verifiableCredential": credential,
          "options": {
            "verificationMethod": format!("{}#{}", issuer.did, issuer.did.chars().skip(8).collect::<String>()),
            "proofPurpose": "assertionMethod"
          }
        });

        // Act
        let response = client
            .post(format!(
                "http://127.0.0.1:{}/verify/credentials",
                issuer.service_address
            ))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    "vc issuer {} failed to verify credential: {}",
                    issuer_did,
                    e
                );
                Error::VcVerifyError
            })?;

        let vr = response.json::<VerifyResult>().await.map_err(|e| {
            tracing::error!("vc issuer {} failed to parse response: {}", issuer_did, e);
            Error::VcIssueError
        })?;

        if vr.errors.len() > 0 {
            tracing::error!(
                "vc issuer {} failed to verify credential: {:?}",
                issuer_did,
                vr.errors
            );
            return Err(Error::VcVerifyError);
        } else {
            tracing::info!("vc issuer {} verified credential", issuer_did);
            return Ok(true);
        }
    }

    pub async fn vc_credential_verify_with_lib(
        pool: &PgPool,
        issuer_did: &str,
        signed_credential: String,
    ) -> Result<bool> {
        let _issuer = VcIssuer::find_by_did(issuer_did, &pool).await?;
        let credential: Value = serde_json::from_str(&signed_credential).map_err(|e| {
            tracing::error!("vc verify {} failed to parse credential: {}", issuer_did, e);
            Error::VcVerifyParserJsonError
        })?;

        let verifiable_credential: VerifiableCredential = serde_json::from_value(credential)?;

        let result = verifiable_credential
            .verify(Some(LinkedDataProofOptions::default()), &DIDKey)
            .await;

        if result.errors.len() > 0 {
            tracing::error!(
                "vc issuer {} failed to verify credential: {:?}",
                issuer_did,
                result.errors
            );
            return Err(Error::VcVerifyError);
        } else {
            tracing::info!("vc issuer {} verified credential", issuer_did);
            return Ok(true);
        }
    }

    /// init pre-defined vc issuer, this should be called when system start
    pub async fn load_predefined_vc_issuers(pool: &PgPool) -> Result<()> {
        let settings = get_configuration().expect("Failed to read configuration.");
        let names: Vec<&str> = settings.did.predefined_issuers.split(",").collect();

        let mut hashed = HashMap::new();
        for name in names {
            hashed.insert(name.to_string(), false);
        }

        match VcIssuer::find_by_names(settings.did.predefined_issuers.split(",").collect(), &pool)
            .await
        {
            Ok(issuers) => {
                // start issuers
                for issuer in issuers {
                    CredentialService::vc_issuer_service_run(&pool, &issuer.did, false).await?;
                    hashed.remove(&issuer.name);
                }
            }
            Err(e) => {
                tracing::info!("No predefined issuers found: {}", e);
            }
        }

        tracing::info!("predefined issuers left: {:?}", hashed.keys());

        for name in hashed.keys() {
            tracing::info!("try to create issuer: {}", name);
            let did = DidService::did_create(pool).await?;
            CredentialService::vc_issuer_create(pool, &did, name).await?;
            CredentialService::vc_issuer_service_run(pool, &did, true).await?;
        }

        Ok(())
    }

    /// use predefined vc issuer to issue credential
    pub async fn vc_credential_issue_predefined(
        pool: &PgPool,
        issuer_name: &str,
        mut credential: Credential,
    ) -> Result<IssueResult> {
        // check if this issuer is running
        let issuer = VcIssuer::find_by_name(issuer_name, &pool).await?;
        if issuer.status != CredentialServiceStatus::Running as i32 {
            tracing::info!("Issuer {} not running, try to start...", issuer_name);
            CredentialService::vc_issuer_service_run(&pool, &issuer.did, true).await?;
        }

        // override issuer did
        credential.issuer_did = issuer.did.clone();
        CredentialService::vc_credential_issue(pool, credential).await
    }
}
