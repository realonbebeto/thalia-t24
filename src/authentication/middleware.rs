use crate::authentication::schemas::AccessLevel;
use crate::authentication::{CustomerSession, SessionType, schemas::SecretKey};
use crate::authentication::{StaffSession, validate_access_token};
use crate::base::error::{BaseError, ErrorExt};
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

const LOGGED_OUT: &str = "You are not logged in. Please log in...";
const DEFAULT_WWW: HeaderValue = HeaderValue::from_static("Basic realm=\"thalia\"");

#[tracing::instrument(name = "Customer Authorization Check" skip(req, next, secret))]
pub async fn reject_unauthorized_customer(
    secret: web::Data<SecretKey>,
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, actix_web::Error> {
    let session = {
        let (http_req, payload) = req.parts_mut();
        CustomerSession::from_request(http_req, payload).await?
    };

    match (
        session.get_sesh_user().to_internal()?,
        validate_access_token(&req, &secret.get_ref().0),
    ) {
        (Some(metadata), Ok(_)) | (Some(metadata), Err(_)) | (None, Ok(metadata)) => {
            // Handle case if not customer
            match metadata.get_persissions() {
                AccessLevel::Customer => {
                    req.extensions_mut().insert(metadata);
                    let mut res = next.call(req).await?;

                    // Add default header
                    let headers = res.headers_mut();
                    headers.insert(header::WWW_AUTHENTICATE, DEFAULT_WWW);

                    return Ok(res.map_body(|_, body| EitherBody::left(body)));
                }
                _ => Err(BaseError::Forbidden)?,
            }
        }
        (None, Err(_)) => Err(BaseError::InvalidCredentials {
            message: LOGGED_OUT.into(),
        })?,
    }
}

#[tracing::instrument(name = "Staff Authorization Check" skip(req, next, secret))]
pub async fn reject_unauthorized_staff(
    secret: web::Data<SecretKey>,
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, actix_web::Error> {
    let session = {
        let (http_req, payload) = req.parts_mut();
        StaffSession::from_request(http_req, payload).await?
    };

    match (
        session.get_sesh_user().to_internal()?,
        validate_access_token(&req, &secret.get_ref().0),
    ) {
        (Some(metadata), Ok(_)) | (Some(metadata), Err(_)) | (None, Ok(metadata)) => {
            // Handle case if not superuser/manager
            match metadata.get_persissions() {
                AccessLevel::Manager | AccessLevel::Superuser => {
                    req.extensions_mut().insert(metadata);
                    let mut res = next.call(req).await?;

                    // Add default header
                    let headers = res.headers_mut();
                    headers.insert(header::WWW_AUTHENTICATE, DEFAULT_WWW);

                    return Ok(res.map_body(|_, body| EitherBody::left(body)));
                }
                _ => Err(BaseError::Forbidden)?,
            }
        }
        (None, Err(_)) => Err(BaseError::InvalidCredentials {
            message: LOGGED_OUT.into(),
        })?,
    }
}
