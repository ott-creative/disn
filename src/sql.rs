use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::Result,
    model::{
        CreateUserData, CreateUserVcTpltSelectionData, CreateVcTpltData, User, UserVcTpltSelection,
        VcTplt,
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

impl VcTplt {
    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<VcTplt> {
        let sql = format!("SELECT * FROM {} WHERE id = $1 LIMIT 1", VcTplt::TABLE);
        Ok(sqlx::query_as(&sql).bind(id).fetch_one(pool).await?)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<VcTplt> {
        let sql = format!("SELECT * FROM {} WHERE name = $1 LIMIT 1", VcTplt::TABLE);
        Ok(sqlx::query_as(&sql).bind(name).fetch_one(pool).await?)
    }

    pub async fn create(data: CreateVcTpltData, pool: &PgPool) -> Result<VcTplt> {
        let sql = format!(
            "
            INSERT INTO {} (name, purpose, fields, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            ",
            VcTplt::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.name)
            .bind(data.purpose)
            .bind(data.fields)
            .bind(data.created_at)
            .bind(data.updated_at)
            .fetch_one(pool)
            .await?)
    }
}

impl UserVcTpltSelection {
    pub async fn find_by_user_id(id: Uuid, pool: &PgPool) -> Result<UserVcTpltSelection> {
        let sql = format!(
            "SELECT * FROM {} WHERE user_id = $1 LIMIT 1",
            UserVcTpltSelection::TABLE
        );
        Ok(sqlx::query_as(&sql).bind(id).fetch_one(pool).await?)
    }

    pub async fn create(data: CreateUserVcTpltSelectionData, pool: &PgPool) -> Result<VcTplt> {
        let sql = format!(
            "
            INSERT INTO {} (user_id, tplt_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            ",
            UserVcTpltSelection::TABLE
        );
        Ok(sqlx::query_as(&sql)
            .bind(data.user_id)
            .bind(data.tplt_id)
            .bind(data.created_at)
            .bind(data.updated_at)
            .fetch_one(pool)
            .await?)
    }
}
