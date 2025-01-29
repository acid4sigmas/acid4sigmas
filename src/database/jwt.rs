use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Pool, Postgres};

use super::Database;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct AuthTokens {
    pub uid: i64,
    pub exp: i64,
    pub jti: String,
}

pub struct AuthTokensDb {
    pool: Pool<Postgres>,
}

#[async_trait::async_trait]
impl Database<AuthTokens> for AuthTokensDb {
    async fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    async fn create_table(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS auth_tokens (
            jti TEXT PRIMARY KEY,
            uid BIGINT NOT NULL,
            expires_at BIGINT NOT NULL
        )
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn fetch(&self) -> Result<Vec<AuthTokens>, sqlx::Error> {
        let auth_tokens = sqlx::query_as::<_, AuthTokens>("SELECT * FROM auth_tokens")
            .fetch_all(&self.pool)
            .await?;
        Ok(auth_tokens)
    }

    async fn fetch_by_key(
        &self,
        key: &str,
        value: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Vec<AuthTokens>, sqlx::Error> {
        let query = match key {
            "uid" => format!("SELECT * FROM auth_tokens WHERE uid = $1"),
            _ => return Err(sqlx::Error::RowNotFound),
        };

        // we only need to downcast as i64, since this is the only type we expect
        if let Some(v) = value.downcast_ref::<i64>() {
            let jti_tokens = sqlx::query_as::<_, AuthTokens>(&query)
                .bind(v)
                .fetch_all(&self.pool)
                .await?;
            Ok(jti_tokens)
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }

    async fn insert(&self, model: AuthTokens) -> Result<(), anyhow::Error> {
        let existing_token_identifier = self
            .fetch_by_key("jti", Box::new(model.jti.clone()))
            .await?;

        if !existing_token_identifier.is_empty() {
            return Err(anyhow::anyhow!("JTI '{}' already exists", model.jti));
        }

        let existing_uid = self.fetch_by_key("uid", Box::new(model.uid)).await?;

        if !existing_uid.is_empty() {
            return Err(anyhow::anyhow!("UID '{}' already exists", model.uid));
        }

        let mut txn = self.pool.begin().await?;

        sqlx::query(
            r#"
        INSERT INTO auth_tokens (jti, uid, expires_at)
        VALUES ($1, $2, $3)
        "#,
        )
        .bind(model.jti)
        .bind(model.uid)
        .bind(model.exp)
        .execute(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(())
    }
}
