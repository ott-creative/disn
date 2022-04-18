use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::Result,
    model::{
        CreateDidData, CreateUserData, CreateVcIssuerData, Did, TxRecord, UpdateVcIssuerData, User,
        VcIssuer,
    },
};

impl TxRecord {
    pub async fn create(tx_hash: String, pool: PgPool) -> Result<TxRecord> {
        let sql = format!(
            "
            INSERT INTO {} (tx_hash, created_at, updated_at)
            VALUES ($1, $2, $3)
            RETURNING *
            ",
            TxRecord::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(tx_hash)
            .bind(Utc::now())
            .bind(Utc::now())
            .fetch_one(&pool)
            .await?)
    }

    pub async fn find_by_send_status(send_status: i32, pool: PgPool) -> Result<Vec<TxRecord>> {
        let sql = format!(
            "SELECT * FROM {} WHERE send_status = {}",
            TxRecord::TABLE,
            send_status
        );
        Ok(sqlx::query_as(&sql).fetch_all(&pool).await?)
    }

    pub async fn update_send_status(
        tx_hash: String,
        send_status: i32,
        block_number: Option<i64>,
        pool: PgPool,
    ) -> Result<TxRecord> {
        let sql = format!(
            "
            UPDATE {} SET
                send_status = $2,
                block_number = $3,
                updated_at = $4
            WHERE tx_hash = $1
            RETURNING *
            ",
            TxRecord::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(tx_hash)
            .bind(send_status)
            .bind(block_number)
            .bind(Utc::now())
            .fetch_one(&pool)
            .await?)
    }
}

impl User {
    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<User> {
        let sql = format!("SELECT * FROM {} WHERE id = $1 LIMIT 1", User::TABLE);
        Ok(sqlx::query_as(&sql).bind(id).fetch_one(pool).await?)
    }

    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<User> {
        let sql = format!("SELECT * FROM {} WHERE email = $1 LIMIT 1", User::TABLE);
        Ok(sqlx::query_as(&sql).bind(email).fetch_one(pool).await?)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<User> {
        let sql = format!("SELECT * FROM {} WHERE name = $1 LIMIT 1", User::TABLE);
        Ok(sqlx::query_as(&sql).bind(name).fetch_one(pool).await?)
    }

    pub async fn create(data: CreateUserData, pool: &PgPool) -> Result<User> {
        let sql = format!(
            "
            INSERT INTO {} (name, email, password, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            ",
            User::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.name)
            .bind(data.email)
            .bind(data.password)
            .bind(data.created_at)
            .bind(data.updated_at)
            .fetch_one(pool)
            .await?)
    }
}

impl VcIssuer {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<VcIssuer>> {
        let sql = format!(
            "SELECT * FROM {} ORDER BY service_address DESC",
            VcIssuer::TABLE
        );
        Ok(sqlx::query_as(&sql).fetch_all(pool).await?)
    }

    pub async fn find_by_did(did: &str, pool: &PgPool) -> Result<VcIssuer> {
        let sql = format!("SELECT * FROM {} WHERE did = $1 LIMIT 1", VcIssuer::TABLE);
        Ok(sqlx::query_as(&sql).bind(did).fetch_one(pool).await?)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<VcIssuer> {
        let sql = format!("SELECT * FROM {} WHERE name = $1 LIMIT 1", VcIssuer::TABLE);
        Ok(sqlx::query_as(&sql).bind(name).fetch_one(pool).await?)
    }

    pub async fn find_by_names(names: Vec<&str>, pool: &PgPool) -> Result<Vec<VcIssuer>> {
        let sql = format!(
            "SELECT * FROM {} WHERE name IN ({})",
            VcIssuer::TABLE,
            names
                .iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(",")
        );
        Ok(sqlx::query_as(&sql).fetch_all(pool).await?)
    }

    pub async fn create(data: CreateVcIssuerData, pool: &PgPool) -> Result<VcIssuer> {
        let sql = format!(
            "
            INSERT INTO {} (did, name, service_address, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            ",
            VcIssuer::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.did)
            .bind(data.name)
            .bind(data.service_address)
            .bind(data.created_at)
            .bind(data.updated_at)
            .fetch_one(pool)
            .await?)
    }

    pub async fn update(data: UpdateVcIssuerData, pool: &PgPool) -> Result<VcIssuer> {
        let sql = format!(
            "
            UPDATE {} SET
                service_address = $2,
                status = $3,
                pid = $4,
                updated_at = $5
            WHERE did = $1
            RETURNING *
            ",
            VcIssuer::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.did)
            .bind(data.service_address)
            .bind(data.status)
            .bind(data.pid)
            .bind(data.updated_at)
            .fetch_one(pool)
            .await?)
    }
}

impl Did {
    pub async fn create(data: CreateDidData, pool: &PgPool) -> Result<Did> {
        let sql = format!(
            "
            INSERT INTO {} (id, jwk, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            ",
            Did::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.id)
            .bind(data.jwk)
            .bind(data.created_at)
            .bind(data.updated_at)
            .fetch_one(pool)
            .await?)
    }

    pub async fn find_by_id(id: &str, pool: &PgPool) -> Result<Did> {
        let sql = format!("SELECT * FROM {} WHERE id = $1 LIMIT 1", Did::TABLE);
        Ok(sqlx::query_as(&sql).bind(id).fetch_one(pool).await?)
    }
}

/*
impl PassbaseIdentity {
    pub async fn create(data: CreatePassbaseIdentity, pool: &PgPool) -> Result<PassbaseIdentity> {
        let sql = format!(
            "
            INSERT INTO {} (id, did, identity, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            ",
            PassbaseIdentity::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.id)
            .bind(data.did)
            .bind(data.identity)
            .bind(data.status)
            .bind(data.created_at)
            .bind(data.updated_at)
            .fetch_one(pool)
            .await?)
    }

    pub async fn update(data: PassbaseIdentity, pool: &PgPool) -> Result<PassbaseIdentity> {
        let sql = format!(
            "
            UPDATE {} SET
                identity = $2,
                status = $3,
                updated_at = $4,
                is_adult = $5,
                tx_hash = $6,
                is_backend_notified = $7
            WHERE id = $1
            RETURNING *
            ",
            PassbaseIdentity::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.id)
            .bind(data.identity)
            .bind(data.status)
            .bind(data.updated_at)
            .bind(data.is_adult)
            .bind(data.tx_hash)
            .bind(data.is_backend_notified)
            .fetch_one(pool)
            .await?)
    }

    pub async fn find_by_id(id: &str, pool: &PgPool) -> Result<PassbaseIdentity> {
        let sql = format!(
            "SELECT * FROM {} WHERE id = $1 LIMIT 1",
            PassbaseIdentity::TABLE
        );
        Ok(sqlx::query_as(&sql).bind(id).fetch_one(pool).await?)
    }
}*/
