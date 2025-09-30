use actix_web::http::header::HeaderMap;
use error_stack::{Report, ResultExt};

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("The `Authorization` header is missing")]
    Empty,
    #[error("The `Authorization` header was not a valid UTF8 string.")]
    Invalid,
    #[error("The authorization scheme was not `Bearer`.")]
    BadScheme,
}

#[tracing::instrument(name = "Read request access token", skip(headers))]
pub fn read_request_access_token(headers: &HeaderMap) -> Result<String, Report<HeaderError>> {
    let header_value = headers
        .get("authorization")
        .ok_or(HeaderError::Empty)
        .change_context(HeaderError::Empty)?
        .to_str()
        .change_context(HeaderError::Invalid)?;

    let access_token = header_value
        .strip_prefix("Bearer ")
        .ok_or(HeaderError::BadScheme)
        .change_context(HeaderError::BadScheme)?;

    Ok(access_token.to_string())
}
