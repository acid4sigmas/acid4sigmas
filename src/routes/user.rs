use acid4sigmas_new_models::models::user::CreateUser;
use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use sqlx::{Pool, Postgres};

use crate::services::{user::CreateUserService, Service};
use acid4sigmas_new_models::error::ExtendedResponse;

#[post("/create_user")]
pub async fn create_user(
    body: web::Json<CreateUser>,
    pool: web::Data<Pool<Postgres>>,
) -> impl Responder {
    let body = body.into_inner();
    let result = body.validate();

    if let Err(e) = result {
        return HttpResponse::BadRequest().json(e);
    }

    let service = CreateUserService::new(Some(pool.get_ref().clone()), body.clone());
    match service.run().await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::build(StatusCode::from_u16(e.status_code).unwrap())
            .json(ExtendedResponse::<()>::error(e)), // use ::<():: because no Data can be returned at this point.
    }
}

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users").service(create_user));
}
