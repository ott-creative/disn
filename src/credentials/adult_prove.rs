use serde_json::{json, Value};
use uuid::Uuid;

use super::VerifiableCredential;

pub struct CredentialAdultProve {
    pub identity: String,
    pub is_adult: bool,
}

impl VerifiableCredential for CredentialAdultProve {
    fn generate_unsigned(&self, issuer: &str, holder: &str) -> Value {
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
