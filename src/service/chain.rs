use crate::configuration::{get_configuration, ChainSettings};
use crate::error::Result;
use secp256k1::SecretKey;
use secrecy::ExposeSecret;
use std::str::FromStr;
use std::{fs, io::Read};
use web3::{
    contract::{Contract, Options},
    contract::tokens::Tokenize,
    transports::Http,
    types::Address,
};

pub struct ChainService;
use sqlx::PgPool;
use crate::model::TxRecord;


impl ChainService {
    pub async fn send_tx(pool: &PgPool, contract: &str, func: &str, params: impl Tokenize) -> Result<String> {
        let settings = get_configuration().expect("Failed to get configuration");
        let prvk = SecretKey::from_str(settings.chain.controller_private_key.expose_secret())
            .expect("Failed to parse private key");
        let contract = Self::contract(&settings.chain, contract)?;
        let tx_hash = contract
            .signed_call(
                func,
                params,
                Options::default(),
                &prvk,
            )
            .await?;
        let tx_hash = tx_hash.to_string();
        TxRecord::create(tx_hash.clone(), pool).await?;
        Ok(tx_hash)
    }

    fn contract(config: &ChainSettings, name: &str) -> Result<Contract<Http>> {
        let transport = web3::transports::Http::new(&config.provider)?;
        let web3 = web3::Web3::new(transport);
        let contract_address = Address::from_str(
            config
                .get_contract_address(name)
                .expect("Failed to locate contract"),
        )
        .expect("Failed to parse contract address");

        let mut abi_file = fs::File::open(format!("{}/{}.json", config.contract_abi_path, name))
            .expect("Failed to open contract ABI");
        let mut abi = String::new();
        abi_file
            .read_to_string(&mut abi)
            .expect("Unable to read the contract abi file");

        Ok(Contract::from_json(
            web3.eth(),
            contract_address,
            abi.as_bytes(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use crate::configuration::get_configuration;

    #[tokio::test]
    async fn send_and_query_tx() {
        let contract_name = "identity".to_string();
        let settings = get_configuration().expect("Failed to get configuration");

        let data = uuid::Uuid::new_v4().to_string();
        let email = format!("{}@example.com", &data);
        let configuration = get_configuration().expect("Failed to read configuration.");
        let pg_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
        let _tx_hash = ChainService::send_tx(&pg_pool, &contract_name, "saveEvidence", (email.clone(), data.clone()))
            .await
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(10));
        let contract = ChainService::contract(&settings.chain, &contract_name).unwrap();
        let result: String = contract
            .query("getEvidence", email, None, Options::default(), None)
            .await
            .unwrap();
        assert_eq!(result, data);
    }
}
