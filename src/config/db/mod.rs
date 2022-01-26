use axum::async_trait;

pub mod postgres;

#[async_trait]
pub trait DbPool {
    async fn retrieve(has_db: bool) -> Self;
}
