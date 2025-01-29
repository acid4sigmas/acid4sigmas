use crate::{
    database::{user::UserDb, Database},
    utils::{
        jsonwebtoken::JsonWebTokenHandler, snowflake_id::generate_uid,
        timestamp::convert_timestamp_to_utc,
    },
};

use super::Service;
use acid4sigmas_new_models::error::{Error, ExtendedResponse, StatusCode};
use acid4sigmas_new_models::models::user::{CreateUser, CreateUserResponse, User};
use chrono::{Duration, Utc};
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct CreateUserService {
    pool: Option<Pool<Postgres>>,
    input: CreateUser,
}

#[async_trait::async_trait]
impl Service<CreateUser, CreateUserResponse> for CreateUserService {
    fn new(pool: Option<Pool<Postgres>>, input: CreateUser) -> Self
    where
        Self: Sized,
    {
        Self { pool, input }
    }

    async fn run(&self) -> Result<ExtendedResponse<CreateUserResponse>, Error> {
        let pool = if let Some(pool) = self.pool.clone() {
            pool
        } else {
            return Err(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database required.",
            ));
        };

        let user_db = UserDb::new(pool.clone()).await;

        user_db
            .create_table()
            .await
            .map_err(|e| Error::new(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        let uid = generate_uid();

        // attempt to insert user into db
        user_db
            .insert(User {
                uid,
                username: self.input.username.clone(),
                nickname: self.input.nickname.clone(),
            })
            .await
            .map_err(|e| Error::new(StatusCode::CONFLICT, &e.to_string()))?;

        let exp = (Utc::now() + Duration::hours(1)).timestamp();

        let token_handler = JsonWebTokenHandler::new(pool).await;

        let token = token_handler
            .create_jwt(uid, Some(exp))
            .await
            .map_err(|e| Error::new(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        let response = ExtendedResponse::success(CreateUserResponse {
            user: User {
                username: self.input.username.clone(),
                nickname: self.input.nickname.clone(),
                uid,
            },
            session_token: token,
            expires_at: convert_timestamp_to_utc(exp),
        });

        Ok(response)
    }
}
