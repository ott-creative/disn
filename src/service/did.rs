use crate::configuration::get_configuration;
use crate::error::{Error, Result};
use chrono::Utc;
use did_method_key::DIDKey;
use didkit::{DIDMethod, Source, JWK};
use sqlx::PgPool;
use std::process::Command;

pub struct DidService;

use crate::model::{CreateDidData, Did};

impl DidService {
    pub async fn did_create(pool: &PgPool) -> Result<String> {
        // create jwk, a static step
        let jwk = JWK::generate_ed25519().map_err(|err| Error::from(err))?;
        // jwk to did-key
        let did = DIDKey
            .generate(&Source::Key(&jwk))
            .ok_or_else(|| Error::DidGenerateError)?;
        // Store did and private key to backend
        // TODO: encrypt
        let jwk_str = serde_json::to_string(&jwk).unwrap();

        let data = CreateDidData {
            id: did.clone(),
            jwk: jwk_str,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Did::create(data, &pool).await?;
        Ok(did)
    }

    pub async fn did_resolve(did: &str) -> Result<String> {
        let configuration = get_configuration().expect("Failed to read configuration.");

        let output = Command::new(format!("{}/didkit", configuration.did.didkit_path))
            .arg("did-resolve")
            .arg(did)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
