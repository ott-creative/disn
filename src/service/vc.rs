use crate::{
    credentials::{
        adult_prove::CredentialAdultProve, personal_identity::CredentialPersonalIdentity,
        VerifiableCredential as VC,
    },
    error::{Error, Result},
    model::{CreateVcIssuerData, UpdateVcIssuerData, VcIssuer},
    service::{
        chain::{ChainCredentialType, CONTRACT_CREDENTIALS_NAME, CONTRACT_REVOKED_NAME},
        did::DidService,
    },
    utils::envelope,
    CONFIG,
};
use chrono::Utc;
use ethabi::FixedBytes;
use serde_json::{json, Value};
use std::{collections::HashMap, process::Command, time::SystemTime};
use uuid::Uuid;
use web3::contract::Options;

use crate::CHAIN;
use did_method_key::DIDKey;
use didkit::{LinkedDataProofOptions, VerifiableCredential, VerifiablePresentation, JWK};

pub struct CredentialService;

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
    pub issuer_cipher: String,
    pub holder_cipher: String,
    pub signed_credential_encrypt: String,
    pub signed_credential_plain: String,
    pub tx_hash: String,
}

pub enum Credentials {
    AdultProve(CredentialAdultProve),
    PersonalIdentity(CredentialPersonalIdentity),
}

impl VC for Credentials {
    fn generate_unsigned(&self, issuer: &str, holder: &str) -> Value {
        match self {
            Credentials::AdultProve(cred) => cred.generate_unsigned(issuer, holder),
            Credentials::PersonalIdentity(cred) => cred.generate_unsigned(issuer, holder),
        }
    }
}

impl Credentials {
    pub fn contract_name(&self) -> &str {
        match self {
            Credentials::AdultProve(_) => "identity",
            Credentials::PersonalIdentity(_) => "identity",
        }
    }
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

fn build_unsigned_vp(holder_did: &str, signed_credential: Value) -> Value {
    json!({
        "@context": ["https://www.w3.org/2018/credentials/v1", "https://credential.codegene.xyz/context/adult.jsonld"],
        "id": format!("urn:uuid:{}", Uuid::new_v4().to_string()),
        "type": ["VerifiablePresentation"],
        "holder": holder_did,
        "verifiableCredential": [signed_credential],
    })
}

impl CredentialService {
    /// create a new vc issuer
    pub async fn vc_issuer_create(did: &str, name: &str) -> Result<()> {
        let data = CreateVcIssuerData {
            did: did.to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            service_address: 0,
            status: CredentialServiceStatus::Created.into(),
        };

        VcIssuer::create(data).await?;

        Ok(())
    }

    /// get available service address(port)
    pub async fn vc_issuer_get_available_port() -> Result<u16> {
        let mut port = CONFIG.did.vc_issuer_port_start;
        let issuers = VcIssuer::find_all().await?;
        if issuers.len() > 0 && issuers[0].service_address > 0 {
            port = (issuers[0].service_address + 1) as u16;
        }

        Ok(port)
    }

    /// get vc issuer by did
    pub async fn vc_issuer_get_by_did(did: &str) -> Result<VcIssuer> {
        VcIssuer::find_by_did(did).await
    }

    /// start vc issuer service
    pub async fn vc_issuer_service_run(did: &str, force_start: bool) -> Result<()> {
        tracing::info!("Try to run vc issuer: {}", did);
        // read vc issuer, check if it's running
        let mut issuer = VcIssuer::find_by_did(did).await?;
        if !force_start && issuer.status == CredentialServiceStatus::Running as i32 {
            tracing::info!("Issuer {} already running", did);
            return Ok(());
        }

        // read out related private key
        let did_instance = Did::find_by_id(did).await?;

        // check if can get available port
        if issuer.service_address == 0 {
            let port = CredentialService::vc_issuer_get_available_port().await?;
            issuer.service_address = port as i32;
        }

        tracing::info!("assign vc issuer port: {}", issuer.service_address);

        // start didkit-http service
        let mut run_didhttp = Command::new(format!("{}/didkit-http", CONFIG.did.didkit_path));
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
                VcIssuer::update(data).await?;
                Ok(())
            }
            Err(e) => {
                tracing::error!("vc issuer {} failed to start: {}", did, e);
                Err(Error::VcIssuerRunningError)
            }
        }
    }

    /// stop vc issuer service
    pub async fn vc_issuer_service_disable(did: &str) -> Result<()> {
        // read vc issuer, check if it's running
        let issuer = VcIssuer::find_by_did(did).await?;
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

        VcIssuer::update(data).await?;

        Ok(())
    }

    /// restart all vc issuers which db status is running
    pub async fn vc_issuer_service_restart() -> Result<()> {
        let issuers = VcIssuer::find_all().await?;
        for issuer in issuers {
            if issuer.status == CredentialServiceStatus::Running as i32 {
                CredentialService::vc_issuer_service_run(&issuer.did, true).await?;
            }
        }

        Ok(())
    }

    /*
        /// issue credential
        pub async fn vc_credential_issue( credential: Credential) -> Result<IssueResult> {
            // check if this issuer is running
            let issuer = VcIssuer::find_by_did(&credential.issuer_did).await?;
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
    */
    //. issue credential with lib access
    pub async fn vc_credential_issue_with_lib(credential: Credential) -> Result<IssueResult> {
        // check if this issuer is running
        // TODO: join query
        let issuer = VcIssuer::find_by_did(&credential.issuer_did).await?;
        let issuer_did = Did::find_by_id(&issuer.did).await?;
        let holder_did = Did::find_by_id(&credential.holder_did).await?;

        let credential_unsigned = credential
            .credential
            .generate_unsigned(&issuer.did, &credential.holder_did);

        let key: JWK = serde_json::from_str(&issuer_did.jwk)?;
        let mut verifiable_credential: VerifiableCredential =
            serde_json::from_value(credential_unsigned)?;
        let proof = verifiable_credential
            .generate_proof(&key, &LinkedDataProofOptions::default(), &DIDKey)
            .await?;

        verifiable_credential.add_proof(proof);

        let signed_credential = serde_json::to_vec(&verifiable_credential)?;
        let signed_credential_str = String::from_utf8(signed_credential)?;

        // encrypt credential
        let encrypted = envelope::seal(
            &issuer_did.encrypt_public_key,
            &holder_did.encrypt_public_key,
            &signed_credential_str,
        )?;

        // submit to chain
        let tx_hash = CHAIN
            .send_tx(
                credential.credential.contract_name(),
                "saveCredential",
                (
                    verifiable_credential.id.unwrap().to_string(),
                    encrypted.0.clone(),
                    vec![holder_did.encrypt_public_key, issuer_did.encrypt_public_key],
                    vec![encrypted.2.clone(), encrypted.1.clone()],
                ),
            )
            .await?;

        Ok(IssueResult {
            signed_credential_encrypt: encrypted.0,
            signed_credential_plain: signed_credential_str,
            issuer_did: issuer.did,
            holder_did: credential.holder_did,
            issuer_cipher: encrypted.1,
            holder_cipher: encrypted.2,
            tx_hash,
        })
    }

    /// revoke credential
    pub async fn vc_credential_revoke(credential_id: &str, issuer_did: &str) -> Result<()> {
        // readout if this credential is already revoked
        let is_revoked =
            CredentialService::is_credential_revoked(credential_id.to_string()).await?;
        if is_revoked {
            // already revoked
            tracing::info!(
                "credential {} already revoked, issuer {}",
                credential_id,
                issuer_did,
            );
            return Ok(());
        }

        // check if this credential is issued by this issuer

        let decrypt = CredentialService::vc_credential_decrypt(issuer_did, credential_id).await?;

        // check if id match
        let credential_json = serde_json::from_str::<VerifiableCredential>(&decrypt)?;
        if !credential_json.id.unwrap().to_string().eq(credential_id) {
            return Err(Error::VcRevokeNotIssuer);
        }

        // perform revoke
        let revoked_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cert_type: [u8; 32] = ChainCredentialType::Identity.into();
        let tx_hash = CHAIN
            .send_tx(
                CONTRACT_REVOKED_NAME,
                "revoke",
                (
                    revoked_time,
                    cert_type,
                    credential_id.to_string(),
                    issuer_did.to_string(),
                ),
            )
            .await?;

        tracing::info!("credential revoke hash: {}", tx_hash);

        Ok(())
    }

    /// decrypt credential
    pub async fn vc_credential_decrypt(did: &str, credential_id: &str) -> Result<String> {
        let controller = Did::find_by_id(did).await?;

        let contract = CHAIN.contract(CONTRACT_CREDENTIALS_NAME)?;
        let (cipher_data, cipher_key): (String, String) = contract
            .query(
                "getCredential",
                (credential_id.to_string(), controller.encrypt_public_key),
                None,
                Options::default(),
                None,
            )
            .await?;

        tracing::info!("vc_credential_decrypt cipher_data: {}", cipher_data);
        if cipher_data.is_empty() {
            return Err(Error::VcCredentialNotFound);
        }

        Ok(envelope::unseal(
            &cipher_data,
            &cipher_key,
            &controller.encrypt_private_key,
        )?)
    }
    /*
        /// verify credential
        pub async fn vc_credential_verify(issuer_did: &str, signed_credential: String) -> Result<bool> {
            // check if this issuer is running
            let issuer = VcIssuer::find_by_did(issuer_did).await?;
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
    */
    pub async fn vc_credential_verify(did: &str, credential_id: &str) -> Result<String> {
        let decrypt = CredentialService::vc_credential_decrypt(did, credential_id).await?;
        let credential: Value = serde_json::from_str(&decrypt).map_err(|e| {
            tracing::error!("vc verify {} failed to parse credential: {}", did, e);
            Error::VcVerifyParserJsonError
        })?;

        let verifiable_credential: VerifiableCredential = serde_json::from_value(credential)?;

        let result = verifiable_credential
            .verify(Some(LinkedDataProofOptions::default()), &DIDKey)
            .await;

        if result.errors.len() > 0 {
            tracing::error!("{} failed to verify credential: {:?}", did, result.errors);
            return Err(Error::VcVerifyError);
        } else {
            tracing::info!("{} verified credential", did);
            // perform chain check to ensure it's not revoked

            let is_revoked = CredentialService::is_credential_revoked(
                verifiable_credential.id.unwrap().to_string(),
            )
            .await?;
            if is_revoked {
                return Err(Error::VcRevokedError);
            } else {
                return Ok("OK".to_string());
            }
        }
    }

    async fn is_credential_revoked(credential_id: String) -> Result<bool> {
        let contract = CHAIN.contract(CONTRACT_REVOKED_NAME).unwrap();
        let (is_revoke_active, _chain_revoked_time, _chain_cert_type, _chain_issuer): (
            bool,
            u64,
            FixedBytes,
            String,
        ) = contract
            .query(
                "getRevokedInfo",
                credential_id,
                None,
                Options::default(),
                None,
            )
            .await?;

        Ok(is_revoke_active)
    }

    /// init pre-defined vc issuer, this should be called when system start
    pub async fn load_predefined_vc_issuers() -> Result<()> {
        let names: Vec<&str> = CONFIG.did.predefined_issuers.split(",").collect();

        let mut hashed = HashMap::new();
        for name in names {
            hashed.insert(name.to_string(), false);
        }

        match VcIssuer::find_by_names(CONFIG.did.predefined_issuers.split(",").collect()).await {
            Ok(issuers) => {
                // start issuers
                for issuer in issuers {
                    // CredentialService::vc_issuer_service_run(&issuer.did, false).await?;
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
            let did = DidService::did_create().await?;
            CredentialService::vc_issuer_create(&did, name).await?;
            // CredentialService::vc_issuer_service_run(&did, true).await?;
        }

        Ok(())
    }

    /// use predefined vc issuer to issue credential
    /*pub async fn vc_credential_issue_predefined(

        issuer_name: &str,
        mut credential: Credential,
    ) -> Result<IssueResult> {
        // check if this issuer is running
        let issuer = VcIssuer::find_by_name(issuer_name).await?;
        if issuer.status != CredentialServiceStatus::Running as i32 {
            tracing::info!("Issuer {} not running, try to start...", issuer_name);
            CredentialService::vc_issuer_service_run(&issuer.did, true).await?;
        }

        // override issuer did
        credential.issuer_did = issuer.did.clone();
        CredentialService::vc_credential_issue_with_lib(credential).await
    }*/

    pub async fn vp_presentation(holder: &str, signed_credential: &str) -> Result<String> {
        let holder_did = Did::find_by_id(holder).await?;

        let credential: Value = serde_json::from_str(signed_credential).map_err(|e| {
            tracing::error!("vc presentation issue failed to parse credential: {}", e);
            Error::VcVerifyParserJsonError
        })?;

        let unsigned_presentation = build_unsigned_vp(holder, credential);

        let key: JWK = serde_json::from_str(&holder_did.jwk)?;
        let mut verifiable_presentation: VerifiablePresentation =
            serde_json::from_value(unsigned_presentation)?;
        let proof = verifiable_presentation
            .generate_proof(&key, &LinkedDataProofOptions::default(), &DIDKey)
            .await?;

        verifiable_presentation.add_proof(proof);

        let signed_presentation = serde_json::to_vec(&verifiable_presentation)?;

        Ok(String::from_utf8(signed_presentation)?)
    }

    pub async fn vp_verify(signed_presentation: String) -> Result<bool> {
        let presentation: Value = serde_json::from_str(&signed_presentation).map_err(|e| {
            tracing::error!("vp verify failed to parse credential: {}", e);
            Error::VcVerifyParserJsonError
        })?;
        let verifiable_presentation: VerifiablePresentation = serde_json::from_value(presentation)?;

        let result = verifiable_presentation
            .verify(Some(LinkedDataProofOptions::default()), &DIDKey)
            .await;

        if result.errors.len() > 0 {
            tracing::error!("vp failed to verify credential: {:?}", result.errors);
            return Err(Error::VpVerifyError);
        } else {
            return Ok(true);
        }
    }
}
