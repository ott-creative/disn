use serde_json::Value;

pub mod adult_prove;
pub mod personal_identity;

pub trait VerifiableCredential {
    fn generate_unsigned(&self, issuer: &str, holder: &str) -> Value;
}
