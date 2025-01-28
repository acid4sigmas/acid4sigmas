use acid4sigmas_new_models::error::{Error, ExtendedResponse};
use sqlx::{Pool, Postgres};
pub mod user;

#[async_trait::async_trait]
pub trait Service<InT, OutT> {
    fn new(pool: Option<Pool<Postgres>>, input: InT) -> Self
    where
        Self: Sized;
    async fn run(&self) -> Result<ExtendedResponse<OutT>, Error>;
}
