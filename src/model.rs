use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub const TABLE: &'static str = "users";
}

#[derive(Debug)]
pub struct CreateUserData {
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Did {
    pub id: String,
    pub jwk: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Did {
    pub const TABLE: &'static str = "dids";
}

#[derive(Debug)]
pub struct CreateDidData {
    pub id: String,
    pub jwk: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct VcIssuer {
    pub did: String,
    pub service_address: i32,
    pub status: i32,
    pub pid: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VcIssuer {
    pub const TABLE: &'static str = "vc_issuers";
}

#[derive(Debug)]
pub struct CreateVcIssuerData {
    pub did: String,
    pub service_address: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct UpdateVcIssuerData {
    pub did: String,
    pub service_address: i32,
    pub pid: Option<i32>,
    pub status: i32,
    pub updated_at: DateTime<Utc>,
}
