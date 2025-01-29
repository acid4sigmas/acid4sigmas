use acid4sigmas_new_models::error::Error as HttpError;
use acid4sigmas_new_models::error::StatusCode;

use actix_web::http::StatusCode as ActixStatusCode;
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse},
    http::header::AUTHORIZATION,
    web, Error, HttpMessage, HttpResponse,
};

use actix_web_lab::middleware::Next;
use serde_json::json;
use sqlx::{Pool, Postgres};

use crate::utils::jsonwebtoken::{self, JsonWebTokenHandler};

pub async fn check_auth_mw<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<BoxBody>, Error>
where
    B: MessageBody + 'static,
{
    // TODO: Cache implementation.
    let _path = req.path().to_string();

    // get the database pool
    let db_pool = match req.app_data::<web::Data<Pool<Postgres>>>() {
        Some(pool) => pool,
        None => {
            let error = HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get db pool!");

            let http_res =
                HttpResponse::build(ActixStatusCode::from_u16(error.status_code).unwrap())
                    .json(error.error)
                    .map_into_boxed_body();

            let (req, _pl) = req.into_parts();

            return Ok(ServiceResponse::new(req, http_res));
        }
    };

    if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
        let auth_header = auth_header.to_str().unwrap();

        match JsonWebTokenHandler::new(db_pool.get_ref().clone())
            .await
            .validate_jwt(auth_header)
            .await
        {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
            }
            Err(e) => {
                let error = HttpError::new(
                    StatusCode::UNAUTHORIZED,
                    &format!("failed to validate json web token: {:?}", e),
                );

                let http_res =
                    HttpResponse::build(ActixStatusCode::from_u16(error.status_code).unwrap())
                        .json(error.error)
                        .map_into_boxed_body();

                let (req, _pl) = req.into_parts();

                return Ok(ServiceResponse::new(req, http_res));
            }
        }
    } else {
        let error = HttpError::new(StatusCode::UNAUTHORIZED, "Authorization header missing!");

        let http_res = HttpResponse::build(ActixStatusCode::from_u16(error.status_code).unwrap())
            .json(error.error)
            .map_into_boxed_body();

        let (req, _pl) = req.into_parts();

        return Ok(ServiceResponse::new(req, http_res));
    }

    let res = next.call(req).await?;
    Ok(res.map_body(|_, body| BoxBody::new(body)))
}
