use std::str::FromStr;
use web3::{
  contract::{Contract, Options},
  types::Address,
  transports::Http
};
use std::error::Error;
use secp256k1::SecretKey;

static ABI: &'static [u8] = r#"[
  {
    "inputs": [],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "string",
        "name": "key",
        "type": "string"
      },
      {
        "indexed": false,
        "internalType": "string",
        "name": "data",
        "type": "string"
      }
    ],
    "name": "SaveEvidence",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "adminCount",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "name": "admins",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "string",
        "name": "",
        "type": "string"
      }
    ],
    "name": "evidences",
    "outputs": [
      {
        "internalType": "string",
        "name": "",
        "type": "string"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "admin",
        "type": "address"
      },
      {
        "internalType": "bool",
        "name": "isActive",
        "type": "bool"
      }
    ],
    "name": "setAdmin",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "string",
        "name": "key",
        "type": "string"
      },
      {
        "internalType": "string",
        "name": "data",
        "type": "string"
      }
    ],
    "name": "saveEvidence",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "string",
        "name": "key",
        "type": "string"
      }
    ],
    "name": "getEvidence",
    "outputs": [
      {
        "internalType": "string",
        "name": "",
        "type": "string"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  }
]"#.as_bytes();

pub struct ChainService;

impl ChainService {
  pub async fn send_tx(email: &str, data: &str) -> Result<String, Box<dyn Error>> {
    let prvk = SecretKey::from_str("B9A4BD1539C15BCC83FA9078FE89200B6E9E802AE992F13CD83C853F16E8BED4").unwrap();
    let contract = Self::contract()?;
    let tx_hash = contract.signed_call("saveEvidence", (email.to_owned(), data.to_owned()), Options::default(), &prvk).await?;
    Ok(tx_hash.to_string())
  }

  fn contract() -> Result<Contract<Http>, Box<dyn Error>> {
    let transport = web3::transports::Http::new("http://159.203.24.45:8545")?;
    let web3 = web3::Web3::new(transport);
    let contract_address = Address::from_str("0xA3211777B08D672f190aD4aA06172f2EA4350EB9").unwrap();
    Ok(Contract::from_json(web3.eth(), contract_address, ABI)?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn send_and_query_tx() {
    let email = String::from("test@163.com");
    let data = String::from("xxxx");
    let _tx_hash = ChainService::send_tx(&email, &data).await.unwrap();
    std::thread::sleep(std::time::Duration::from_secs(10));
    let contract = ChainService::contract().unwrap();
    let result: String = contract.query("getEvidence", (email), None, Options::default(), None).await.unwrap();
    assert_eq!(result, data);
  }
}