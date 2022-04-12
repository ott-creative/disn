use crate::configuration::{get_configuration, ChainSettings};
use crate::error::Result;
use secp256k1::SecretKey;
use secrecy::ExposeSecret;
use std::str::FromStr;
use std::{fs, io::Read};
use web3::{
    error,
    api::Eth,
    types::{H256, U64},
    contract::{Contract, Options},
    contract::tokens::Tokenize,
    transports::Http,
    types::Address,
    Transport
};
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use sqlx::PgPool;
use crate::model::TxRecord;
use tokio::time::{sleep, Duration as TokioDuration};
use async_recursion::async_recursion;

pub struct ChainService {
    pool: PgPool,
    tx: Sender<(String, PgPool)>,
    settings: ChainSettings,
    web3: web3::Web3<Http>
}

impl ChainService {
    pub async fn send_tx(&self, contract: &str, func: &str, params: impl Tokenize) -> Result<String> {
        let prvk = SecretKey::from_str(self.settings.controller_private_key.expose_secret())
        .expect("Failed to parse private key");
        let contract = self.contract(contract)?;
        let tx_hash = contract
            .signed_call(
                func,
                params,
                Options::default(),
                &prvk,
            )
            .await?;
        let tx_hash = format!("{:#x}", tx_hash);
        self.tx.send((tx_hash.clone(), self.pool.clone()))?;
        TxRecord::create(tx_hash.clone(), self.pool.clone()).await?;
        Ok(tx_hash)
    }

    pub async fn run_confirm_server(pool: PgPool) -> ChainService {
        let (tx, rx): (Sender<(String, PgPool)>, Receiver<(String, PgPool)>) = mpsc::channel();
        let (web3, settings) = Self::web3_config();
        let confirm_server = ChainService{ pool, tx, settings: settings , web3};
        thread::spawn(move || loop {
            let (tx_hash, pgpool) = rx.recv().unwrap();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.spawn(async move {
                ChainService::confirm_tx(pgpool, tx_hash).await;
            });
        });
        confirm_server.confirm_pending_txs().await;
        confirm_server
    }

    async fn confirm_pending_txs(&self) {
        let records = TxRecord::find_by_send_status(0, self.pool.clone()).await.unwrap();
        records.into_iter().for_each(|record|  {
            self.tx.send((record.tx_hash, self.pool.clone())).unwrap();
        });
    }

    #[async_recursion]
    async fn confirm_tx(pool: PgPool, tx_hash: String) {
        let (web3, _) = Self::web3_config();
        let poll_interval = Duration::from_secs(2);
        let tx_hash_256 = H256::from_str(&tx_hash).unwrap();
        let eth = web3.eth();
        let confirmation_check = || Self::tx_receipt_check(&eth, tx_hash_256);
        let result = web3.wait_for_confirmations(poll_interval, 0, confirmation_check).await;
        if result.is_err() {
            Self::retry_confirm(pool.clone(), tx_hash.clone()).await;
        }
        let receipt_result = eth.transaction_receipt(tx_hash_256).await;
        let mut send_status: i32 = 1;
        if let Ok(Some(receipt)) = receipt_result {
            if receipt.status == Some(0.into()) {
                send_status = -1;
            }
        } else {
            Self::retry_confirm(pool.clone(), tx_hash.clone()).await;
        };
        TxRecord::update_send_status(tx_hash, send_status, pool).await.unwrap();
    }

    async fn retry_confirm(pool: PgPool, tx_hash: String) {
        sleep(TokioDuration::from_secs(10)).await;
        Self::confirm_tx(pool, tx_hash).await;
    }

    async fn tx_receipt_check<T: Transport>(eth: &Eth<T>, hash: H256) -> error::Result<Option<U64>> {
        let receipt = eth.transaction_receipt(hash).await?;
        Ok(receipt.and_then(|receipt| receipt.block_number))
    }

    fn web3_config() -> (web3::Web3<Http>, ChainSettings){
        let settings = get_configuration().expect("Failed to get configuration");

        let transport = web3::transports::Http::new(&settings.chain.provider).unwrap();
        (web3::Web3::new(transport), settings.chain)
    }

    fn contract(&self, name: &str) -> Result<Contract<Http>> {
        let contract_address = Address::from_str(
            self.settings
                .get_contract_address(name)
                .expect("Failed to locate contract"),
        )
        .expect("Failed to parse contract address");

        let mut abi_file = fs::File::open(format!("{}/{}.json", &self.settings.contract_abi_path, name))
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
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn send_and_query_tx() {
        let contract_name = "identity".to_string();
        let data = uuid::Uuid::new_v4().to_string();
        let email = format!("{}@example.com", &data);
        let configuration = get_configuration().expect("Failed to read configuration.");
        let pg_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
        let server = ChainService::run_confirm_server(pg_pool).await;
        let _tx_hash = server.send_tx(&contract_name, "saveEvidence", (email.clone(), data.clone()))
            .await
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(60));
        let contract = server.contract(&contract_name).unwrap();
        let result: String = contract
            .query("getEvidence", email, None, Options::default(), None)
            .await
            .unwrap();
        assert_eq!(result, data);
    }
}
