use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::Result,
    model::{
        CreateDidData, CreateUserData, CreateVcIssuerData, Did, UpdateVcIssuerData, User, VcIssuer,
    },
};

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

    pub async fn create(data: CreateVcIssuerData, pool: &PgPool) -> Result<VcIssuer> {
        let sql = format!(
            "
            INSERT INTO {} (did, service_address, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            ",
            VcIssuer::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.did)
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
                updated_at = $4
            WHERE did = $1
            RETURNING *
            ",
            VcIssuer::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.did)
            .bind(data.service_address)
            .bind(data.status)
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
