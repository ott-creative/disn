use crate::configuration::ChainSettings;
use crate::error::Result;
use crate::model::TxRecord;
use async_recursion::async_recursion;
use secp256k1::SecretKey;
use secrecy::ExposeSecret;
use std::str::FromStr;
use std::time::Duration;
use std::{fs, io::Read};
use tokio::time::{sleep, Duration as TokioDuration};
use web3::{
    api::Eth,
    contract::tokens::Tokenize,
    contract::{Contract, Options},
    error,
    transports::Http,
    types::Address,
    types::{H256, U256, U64},
    Transport,
};

pub const CONTRACT_CREDENTIALS_NAME: &str = "identity";
pub const CONTRACT_REVOKED_NAME: &str = "revoked_cert";

#[derive(Clone)]
pub struct ChainService {
    settings: ChainSettings,
    web3: web3::Web3<Http>,
}

pub enum ChainCredentialType {
    Identity,
}

impl Into<[u8; 32]> for ChainCredentialType {
    fn into(self) -> [u8; 32] {
        match self {
            ChainCredentialType::Identity => {
                let mut cert_type = [0u8; 32];
                let cert_type_bytes = "identity".as_bytes();
                cert_type[..cert_type_bytes.len()].copy_from_slice(cert_type_bytes);
                cert_type
            }
        }
    }
}

impl From<String> for ChainCredentialType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "identity" => ChainCredentialType::Identity,
            _ => panic!("Unknown credential type"),
        }
    }
}

impl ChainCredentialType {
    pub fn contract_name(&self) -> String {
        match self {
            ChainCredentialType::Identity => "identity".to_string(),
        }
    }
}

impl ChainService {
    pub async fn send_tx(
        &'static self,
        contract: &str,
        func: &str,
        params: impl Tokenize,
    ) -> Result<String> {
        let prvk = SecretKey::from_str(self.settings.controller_private_key.expose_secret())
            .expect("Failed to parse private key");
        let contract = self.contract(contract)?;
        let mut options = Options::default();
        options.gas = Some(U256::from(2_000_000u64));
        let tx_hash = contract.signed_call(func, params, options, &prvk).await?;
        let tx_hash = format!("{:#x}", tx_hash);
        TxRecord::create(tx_hash.clone()).await?;
        let tx = tx_hash.clone();
        tokio::spawn(async move {
            self.confirm_tx(tx).await;
        });
        Ok(tx_hash)
    }

    pub fn run_confirm_server(settings: ChainSettings) -> ChainService {
        let transport = web3::transports::Http::new(&settings.provider).unwrap();
        let web3 = web3::Web3::new(transport);
        ChainService { settings, web3 }
    }

    #[async_recursion]
    pub async fn confirm_tx(&self, tx_hash: String) {
        let tx_hash_256 = H256::from_str(&tx_hash).unwrap();
        let eth = self.web3.eth();
        let poll_interval = Duration::from_secs(2);
        let confirmation_check = || Self::tx_receipt_check(&eth, tx_hash_256);
        let result = self
            .web3
            .wait_for_confirmations(poll_interval, 0, confirmation_check)
            .await;
        if result.is_err() {
            self.retry_confirm(tx_hash.clone()).await;
        }
        let receipt_result = eth.transaction_receipt(tx_hash_256).await;
        let mut send_status: i32 = -1;
        let mut block_number: Option<i64> = None;
        if let Ok(Some(receipt)) = receipt_result {
            if receipt.status == Some(1u64.into()) {
                send_status = 1;
            }
            if let Some(read_block_number) = receipt.block_number {
                block_number = Some(read_block_number.low_u64() as i64);
            }
        } else {
            self.retry_confirm(tx_hash.clone()).await;
        };
        TxRecord::update_send_status(tx_hash, send_status, block_number)
            .await
            .unwrap();
    }

    async fn retry_confirm(&self, tx_hash: String) {
        sleep(TokioDuration::from_secs(10)).await;
        self.confirm_tx(tx_hash).await;
    }

    async fn tx_receipt_check<T: Transport>(
        eth: &Eth<T>,
        hash: H256,
    ) -> error::Result<Option<U64>> {
        let receipt = eth.transaction_receipt(hash).await?;
        Ok(receipt.and_then(|receipt| receipt.block_number))
    }

    pub fn contract(&self, name: &str) -> Result<Contract<Http>> {
        let contract_address = Address::from_str(
            self.settings
                .get_contract_address(name)
                .expect("Failed to locate contract"),
        )
        .expect("Failed to parse contract address");

        let mut abi_file = fs::File::open(format!(
            "{}/{}.json",
            &self.settings.contract_abi_path, name
        ))
        .expect("Failed to open contract ABI");
        let mut abi = String::new();
        abi_file
            .read_to_string(&mut abi)
            .expect("Unable to read the contract abi file");

        Ok(Contract::from_json(
            self.web3.eth(),
            contract_address,
            abi.as_bytes(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use ethabi::FixedBytes;

    use super::*;
    use crate::CHAIN;

    #[tokio::test]
    async fn test_identity_tx() {
        let contract_name = "identity".to_string();

        let key = uuid::Uuid::new_v4().to_string();
        let cipher_data = key.clone();
        let pub_keys = vec![String::from("pubkey1pubkey1pubkey1pubkey1pubkey1pubkey1")];
        let cipher_keys = vec![String::from("cipherKey1cipherKey1cipherKey1cipherKey1")];
        let tx_hash = CHAIN
            .send_tx(
                &contract_name,
                "saveCredential",
                (
                    key.clone(),
                    cipher_data.clone(),
                    pub_keys.clone(),
                    cipher_keys.clone(),
                ),
            )
            .await
            .unwrap();
        CHAIN.confirm_tx(tx_hash).await;
        let contract = CHAIN.contract(&contract_name).unwrap();
        let (active_cipher_data, active_cipher_key): (String, String) = contract
            .query(
                "getCredential",
                (key, pub_keys[0].clone()),
                None,
                Options::default(),
                None,
            )
            .await
            .unwrap();
        assert_eq!(cipher_data, active_cipher_data);
        assert_eq!(active_cipher_key, cipher_keys[0]);
    }

    use std::time::SystemTime;

    #[tokio::test]
    async fn test_revoked_cert_tx() {
        let contract_name = "revoked_cert".to_string();

        let cert_no = uuid::Uuid::new_v4().to_string();
        let revoked_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cert_type: [u8; 32] = ChainCredentialType::Identity.into();
        let issuer = String::from("issuerissuerissuerissuerissuerissuer");
        let tx_hash = CHAIN
            .send_tx(
                &contract_name,
                "revoke",
                (revoked_time, cert_type, cert_no.clone(), issuer.clone()),
            )
            .await
            .unwrap();
        CHAIN.confirm_tx(tx_hash).await;
        let contract = CHAIN.contract(&contract_name).unwrap();
        let (chain_is_active, chain_revoked_time, chain_cert_type, chain_issuer): (
            bool,
            u64,
            FixedBytes,
            String,
        ) = contract
            .query("getRevokedInfo", cert_no, None, Options::default(), None)
            .await
            .unwrap();
        assert_eq!(true, chain_is_active);
        assert_eq!(revoked_time, chain_revoked_time);
        assert_eq!(
            "identity",
            String::from_utf8(chain_cert_type)
                .unwrap()
                .trim_matches(char::from(0))
        );
        assert_eq!(issuer, chain_issuer);
    }
}
