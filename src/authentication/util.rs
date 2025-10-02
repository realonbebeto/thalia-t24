use crate::base::error::BaseError;
use actix_web::{HttpRequest, http::header::HeaderMap};
use error_stack::ResultExt;
use jsonwebtoken::errors::ErrorKind;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tracing::instrument(name = "Read request access token", skip(headers))]
pub fn read_request_access_token(headers: &HeaderMap) -> Result<String, ErrorKind> {
    let header_value = headers
        .get("authorization")
        .ok_or(ErrorKind::InvalidToken)?
        .to_str()
        .map_err(|_| ErrorKind::InvalidToken)?;

    let access_token = header_value
        .strip_prefix("Bearer ")
        .ok_or(ErrorKind::InvalidToken)?;

    Ok(access_token.to_string())
}

#[tracing::instrument(name = "Read request access token", skip(req))]
pub fn read_request_refresh_token(req: &HttpRequest) -> Result<String, ErrorKind> {
    let refresh_token = req
        .cookie("refresh_token")
        .ok_or(ErrorKind::InvalidToken)?
        .to_string();

    let refresh_token = refresh_token
        .strip_prefix("refresh_token")
        .ok_or(ErrorKind::InvalidToken)?;

    Ok(refresh_token.to_string())
}

pub fn to_unix_expiry(expiry: u64) -> Result<usize, BaseError> {
    let exp = (SystemTime::now() + Duration::from_secs(expiry))
        .duration_since(UNIX_EPOCH)
        .change_context(BaseError::Internal)?
        .as_secs() as usize;

    Ok(exp)
}
