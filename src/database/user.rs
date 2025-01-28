use acid4sigmas_new_models::models::user::User;
use sqlx::{Pool, Postgres};

use super::Database;

pub struct UserDb {
    pool: Pool<Postgres>,
}

#[async_trait::async_trait]
impl Database<User> for UserDb {
    async fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    async fn create_table(&self) -> Result<(), sqlx::Error> {
        let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS users (
            uid BIGINT PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            nickname VARCHAR(255) NOT NULL
        );"#;

        sqlx::query(create_table_query).execute(&self.pool).await?;

        Ok(())
    }

    async fn fetch(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    // in this case we dont need to fetch from multiple keys
    // both username and user id are unique identifiers.
    async fn fetch_by_key(
        &self,
        key: &str,
        value: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Vec<User>, sqlx::Error> {
        let query = match key {
            "username" => format!("SELECT * FROM users WHERE username = $1"),
            "uid" => format!("SELECT * FROM users WHERE uid = $1"),
            _ => return Err(sqlx::Error::RowNotFound), // Unsupported field
        };

        if let Some(v) = value.downcast_ref::<String>() {
            let users = sqlx::query_as::<_, User>(&query)
                .bind(v)
                .fetch_all(&self.pool)
                .await?;
            Ok(users)
        } else if let Some(v) = value.downcast_ref::<i64>() {
            let users = sqlx::query_as::<_, User>(&query)
                .bind(v)
                .fetch_all(&self.pool)
                .await?;
            Ok(users)
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }

    async fn insert(&self, model: User) -> Result<(), anyhow::Error> {
        let existing_user = self
            .fetch_by_key("username", Box::new(model.username.clone()))
            .await?;
        if !existing_user.is_empty() {
            return Err(anyhow::anyhow!(
                "Username '{}' already exists",
                model.username
            ));
        }

        let existing_users = self.fetch_by_key("uid", Box::new(model.uid)).await?;
        if !existing_users.is_empty() {
            return Err(anyhow::anyhow!("ID '{}' already exists", model.uid));
        }

        let query = r#"
        INSERT INTO users (uid, username, nickname)
        VALUES ($1, $2, $3)
        "#;

        // use transaction
        let mut txn = self.pool.begin().await?;

        sqlx::query(query)
            .bind(model.uid)
            .bind(model.username)
            .bind(model.nickname)
            .execute(&mut *txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }
}
