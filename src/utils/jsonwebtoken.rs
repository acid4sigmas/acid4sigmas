use anyhow::anyhow;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sqlx::{Pool, Postgres};

use crate::database::{
    jwt::{AuthTokens, AuthTokensDb},
    Database,
};

const SECRET: &str = "very_SECRETT_keyy!";

pub struct JsonWebTokenHandler {
    db: AuthTokensDb,
}

impl JsonWebTokenHandler {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        Self {
            db: AuthTokensDb::new(pool).await,
        }
    }

    pub async fn create_jwt(&self, user_id: i64, exp: Option<i64>) -> anyhow::Result<String> {
        let exp = if let Some(exp) = exp {
            exp
        } else {
            (Utc::now() + Duration::hours(1)).timestamp() // if None, we default to 1 hour
        };

        let claims = AuthTokens {
            uid: user_id,
            exp,
            jti: uuid::Uuid::new_v4().to_string(),
        };

        // a variable makes sense here
        let db = &self.db;

        db.create_table().await?;
        db.insert(claims.clone()).await?;

        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )?)
    }

    pub async fn validate_jwt(&self, token: &str) -> anyhow::Result<AuthTokens> {
        match decode::<AuthTokens>(
            token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token_data) => {
                let claims = token_data.claims;

                let jtis = &self.db.fetch_by_key("uid", Box::new(claims.uid)).await?;

                let mut is_jti_valid = false;

                // now we iterate over the json token identifiers
                // and compare if the identifier matches the identifier stored in our db
                for jti in jtis {
                    if jti.jti == claims.jti {
                        is_jti_valid = true;
                        break;
                    }
                }

                // check if the token is expired
                if claims.exp < Utc::now().timestamp() {
                    return Err(anyhow!("your token is expired"));
                }

                if is_jti_valid {
                    Ok(claims)
                } else {
                    Err(anyhow!("No valid jti found for this token"))
                }
            }
            Err(_) => Err(anyhow!("failed to validate token")),
        }
    }
}
