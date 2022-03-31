use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use secrecy::ExposeSecret;
use uuid::Uuid;

//use crate::{config::env::JWT_SECRET, error::Result};
use crate::{configuration::get_configuration, error::Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn _new(id: Uuid) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::hours(24);

        Self {
            sub: id,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

pub fn _sign(id: Uuid) -> Result<String> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::_new(id),
        &EncodingKey::from_secret(configuration.security.jwt_secret.expose_secret().as_bytes()),
    )?)
}

pub fn _verify(token: &str) -> Result<Claims> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    Ok(jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(configuration.security.jwt_secret.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)?)
}
