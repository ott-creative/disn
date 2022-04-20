use crate::error::{Error, Result};
use crate::utils::envelope;
use crate::CONFIG;
use chrono::Utc;
use did_method_key::DIDKey;
use didkit::{DIDMethod, Source, JWK};
use std::process::Command;

pub struct DidService;

use crate::model::{CreateDidData, Did};

impl DidService {
    pub async fn did_create() -> Result<String> {
        // create jwk, a static step
        let jwk = JWK::generate_ed25519().map_err(|err| Error::from(err))?;
        // jwk to did-key
        let did = DIDKey
            .generate(&Source::Key(&jwk))
            .ok_or_else(|| Error::DidGenerateError)?;
        // Store did and private key to backend
        // TODO: encrypt
        let jwk_str = serde_json::to_string(&jwk).unwrap();

        // create encrypt keys
        let key_pair = envelope::generate_rsa_keypair();

        let data = CreateDidData {
            id: did.clone(),
            jwk: jwk_str,
            encrypt_public_key: key_pair.1,
            encrypt_private_key: key_pair.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Did::create(data).await?;
        Ok(did)
    }

    pub async fn did_resolve(did: &str) -> Result<String> {
        let output = Command::new(format!("{}/didkit", CONFIG.did.didkit_path))
            .arg("did-resolve")
            .arg(did)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
