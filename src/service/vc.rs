use crate::{
    configuration::get_configuration,
    error::{Error, Result},
    model::{CreateVcIssuerData, UpdateVcIssuerData, VcIssuer},
};
use chrono::Utc;
use did_method_key::DIDKey;
use didkit::{DIDMethod, Source, JWK};
use sqlx::PgPool;
use std::process::Command;

pub struct CredentialService;

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

use crate::model::Did;

impl CredentialService {
    /// create a new vc issuer
    pub async fn vc_issuer_create(pool: &PgPool, did: &str) -> Result<()> {
        let data = CreateVcIssuerData {
            did: did.to_string(),
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
        if issuers.len() > 0 {
            port = (issuers[0].service_address + 1) as u16;
        }

        Ok(port)
    }

    /// start vc issuer service
    pub async fn vc_issuer_service_run(pool: &PgPool, did: &str) -> Result<()> {
        // read vc issuer, check if it's running
        let mut issuer = VcIssuer::find_by_did(did, &pool).await?;
        if issuer.status == CredentialServiceStatus::Running as i32 {
            tracing::info!("Issuer {} already running", did);
            return Ok(());
        }

        // read out related private key
        let did_instance = Did::find_by_id(did, &pool).await?;

        // check if can get available port
        if issuer.service_address == 0 {
            let port = CredentialService::vc_issuer_get_available_port(pool).await?;
            issuer.service_address = port as i32;
        }

        // start didkit-http service
        let settings = get_configuration().expect("Failed to read configuration.");
        let output = Command::new(format!("{}/didkit-http", settings.did.didkit_path))
            .arg("-p")
            .arg(issuer.service_address.to_string())
            .arg("-j")
            .arg(did_instance.jwk)
            .output()?;

        tracing::info!(
            "start vc issuer:{}",
            String::from_utf8_lossy(&output.stdout)
        );

        // update vc issuer status
        let data = UpdateVcIssuerData {
            status: CredentialServiceStatus::Running.into(),
            updated_at: Utc::now(),
            did: did.to_string(),
            service_address: issuer.service_address,
        };

        VcIssuer::update(data, &pool).await?;

        Ok(())
    }

    /// stop vc issuer service
    pub async fn vc_issuer_service_disable(pool: &PgPool, did: &str) -> Result<()> {
        // read vc issuer, check if it's running
        let mut issuer = VcIssuer::find_by_did(did, &pool).await?;
        if issuer.status == CredentialServiceStatus::Disabled as i32 {
            tracing::info!("Issuer {} already stopped", did);
            return Ok(());
        }

        // TODO: kill didkit-http pid

        // update vc issuer status
        let data = UpdateVcIssuerData {
            status: CredentialServiceStatus::Disabled.into(),
            updated_at: Utc::now(),
            did: did.to_string(),
            service_address: issuer.service_address,
        };

        VcIssuer::update(data, &pool).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn vc_lists() {}
}
