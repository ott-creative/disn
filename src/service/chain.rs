use crate::configuration::{get_configuration, ChainSettings};
use crate::error::Result;
use secp256k1::SecretKey;
use secrecy::ExposeSecret;
use std::str::FromStr;
use std::{fs, io::Read};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::Address,
};

pub struct ChainService;

impl ChainService {
    pub async fn send_tx(contract: &str, email: &str, data: &str) -> Result<String> {
        let settings = get_configuration().expect("Failed to get configuration");
        let prvk = SecretKey::from_str(settings.chain.controller_private_key.expose_secret())
            .expect("Failed to parse private key");
        let contract = Self::contract(&settings.chain, contract)?;
        let tx_hash = contract
            .signed_call(
                "saveEvidence",
                (email.to_owned(), data.to_owned()),
                Options::default(),
                &prvk,
            )
            .await?;
        Ok(tx_hash.to_string())
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

    #[tokio::test]
    async fn send_and_query_tx() {
        let contract_name = "identity".to_string();
        let settings = get_configuration().expect("Failed to get configuration");

        let data = uuid::Uuid::new_v4().to_string();
        let email = format!("{}@example.com", &data);
        let _tx_hash = ChainService::send_tx(&contract_name, &email, &data)
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
