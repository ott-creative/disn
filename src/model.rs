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
pub struct VcTplt {
    pub id: Uuid,
    pub name: String,
    pub purpose: String,
    pub fields: String, // Dynamic JSON, TODO: check with postgres native json support
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VcTplt {
    pub const TABLE: &'static str = "vc_tplts";
}

#[derive(Debug)]
pub struct CreateVcTpltData {
    pub name: String,
    pub purpose: String,
    pub fields: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct UserVcTpltSelection {
    pub user_id: Uuid,
    pub tplt_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug)]
pub struct CreateUserVcTpltSelectionData {
    pub user_id: Uuid,
    pub tplt_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl UserVcTpltSelection {
    pub const TABLE: &'static str = "user_vc_tplt_selection";
}

