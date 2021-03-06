use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub api_version: String,
    pub database: DatabaseSettings,
    pub server: ServerSettings,
    pub security: SecuritySettings,
    pub did: DidSettings,
    pub passbase: PassbaseSettings,
    pub chain: ChainSettings,
}

#[derive(serde::Deserialize)]
pub struct PassbaseSettings {
    pub secret_api_key: String,
}

#[derive(serde::Deserialize)]
pub struct ServerSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub backend_notify_url: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(serde::Deserialize)]
pub struct SecuritySettings {
    pub jwt_secret: Secret<String>,
}

#[derive(serde::Deserialize)]
pub struct DidSettings {
    pub didkit_path: String,
    pub api_key: String,
    pub vc_issuer_port_start: u16,
    pub predefined_issuers: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ContractMeta {
    pub name: String,
    pub address: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ChainSettings {
    pub controller_private_key: Secret<String>,
    pub contract_abi_path: String,
    pub provider: String,
    pub contracts: Vec<ContractMeta>,
}

impl ChainSettings {
    pub fn get_contract_address(&self, name: &str) -> Option<&str> {
        self.contracts
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.address.as_str())
    }
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Read the "default" configuration file
    settings.merge(config::File::from(configuration_directory.join("base")).required(true))?;

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    // Layer on the environment-specific values.
    settings.merge(
        config::File::from(configuration_directory.join(environment.as_str())).required(true),
    )?;

    // Add in settings from environment variables (with a prefix of APP and '__' as separator)
    // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
    settings.merge(config::Environment::with_prefix("app").separator("__"))?;

    settings.try_into()
}

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
