use actix_web::HttpMessage;
use actix_web::{
    FromRequest,
    body::{EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    http::header::HeaderValue,
    middleware::Next,
    web,
};

use crate::authentication::{CustomerSession, SessionType, StaffSession};
use crate::base::error::AuthError;
use crate::config::state::AppState;
use crate::user::models::AccessRole;

const DEFAULT_WWW: HeaderValue = HeaderValue::from_static("Basic realm=\"thalia\"");

#[tracing::instrument(name = "Customer Authorization Check" skip(req, next, app_state))]
pub async fn reject_unauthorized_customer(
    app_state: web::Data<AppState>,
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, actix_web::Error> {
    let session = {
        let (http_req, payload) = req.parts_mut();
        CustomerSession::from_request(http_req, payload).await?
    };

    match (
        session.get_sesh_user()?,
        app_state.token_handler.verify_from_service_req(&req),
    ) {
        (Some(metadata), Ok(_)) | (Some(metadata), Err(_)) | (None, Ok(metadata)) => {
            // Handle case if not customer
            match metadata.get_role() {
                AccessRole::Customer => {
                    req.extensions_mut().insert(metadata);
                    let mut res = next.call(req).await?;

                    // Add default header
                    let headers = res.headers_mut();
                    headers.insert(header::WWW_AUTHENTICATE, DEFAULT_WWW);

                    return Ok(res.map_body(|_, body| EitherBody::left(body)));
                }
                _ => Err(AuthError::InsufficientPermissions)?,
            }
        }
        (None, Err(_)) => Err(AuthError::InvalidCredentials(
            "No active session found or missing credentials".into(),
        ))?,
    }
}

#[tracing::instrument(name = "Staff Authorization Check" skip(req, next, app_state))]
pub async fn reject_unauthorized_staff(
    app_state: web::Data<AppState>,
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, actix_web::Error> {
    let session = {
        let (http_req, payload) = req.parts_mut();
        StaffSession::from_request(http_req, payload).await?
    };

    match (
        session.get_sesh_user()?,
        app_state.token_handler.verify_from_service_req(&req),
    ) {
        (Some(metadata), Ok(_)) | (Some(metadata), Err(_)) | (None, Ok(metadata)) => {
            // Handle case if not superuser/manager
            match metadata.get_role() {
                AccessRole::Manager | AccessRole::Superuser => {
                    req.extensions_mut().insert(metadata);
                    let mut res = next.call(req).await?;

                    // Add default header
                    let headers = res.headers_mut();
                    headers.insert(header::WWW_AUTHENTICATE, DEFAULT_WWW);

                    return Ok(res.map_body(|_, body| EitherBody::left(body)));
                }
                _ => Err(AuthError::InsufficientPermissions)?,
            }
        }
        (None, Err(_)) => Err(AuthError::InvalidCredentials(
            "Missing active session or credentials".into(),
        ))?,
    }
}
