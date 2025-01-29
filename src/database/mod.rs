pub mod jwt;
pub mod user;

use sqlx::{Pool, Postgres};

#[async_trait::async_trait]
pub trait Database<T> {
    async fn new(pool: Pool<Postgres>) -> Self
    where
        Self: Sized;
    async fn create_table(&self) -> Result<(), sqlx::Error>;
    async fn fetch(&self) -> Result<Vec<T>, sqlx::Error>;
    async fn fetch_by_key(
        &self,
        key: &str,
        value: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Vec<T>, sqlx::Error>;
    async fn insert(&self, model: T) -> Result<(), anyhow::Error>;
}
