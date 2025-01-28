mod database;
mod middleware;
mod routes;
mod services;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    Pool,
};

use crate::routes::user::user_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = "postgres://postgres:7522@localhost:5432/acid4speed";

    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to create pool");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors.clone()) //cloned it to avoid potential issues
            .app_data(web::Data::new(pool.clone()))
            .configure(user_routes)
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}
