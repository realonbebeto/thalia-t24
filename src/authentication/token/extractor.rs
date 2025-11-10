use crate::base::error::AuthError;
use actix_web::{HttpRequest, cookie::Cookie, dev::ServiceRequest, http::header::HeaderMap};

pub trait RequestLike {
    fn headers(&self) -> &HeaderMap;
    fn cookie(&self, name: &str) -> Option<Cookie<'_>>;
}

impl RequestLike for HttpRequest {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
    fn cookie(&self, name: &str) -> Option<Cookie<'_>> {
        self.cookie(name)
    }
}

impl RequestLike for ServiceRequest {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
    fn cookie(&self, name: &str) -> Option<Cookie<'_>> {
        self.cookie(name)
    }
}

pub trait TokenExtractor {
    fn bearer_token(&self) -> Result<String, AuthError>;
    fn refresh_token(&self) -> Result<String, AuthError>;
    fn refresh_token_from_cookie(&self) -> Result<String, AuthError>;
}

impl<T: RequestLike> TokenExtractor for T {
    fn bearer_token(&self) -> Result<String, AuthError> {
        let header_value = self
            .headers()
            .get("authorization")
            .ok_or(AuthError::MissingAuth("access_token".into()))?
            .to_str()
            .map_err(|_| AuthError::Unauthorized)?;

        let access_token = header_value
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidTokenScheme)?;

        Ok(access_token.into())
    }

    fn refresh_token(&self) -> Result<String, AuthError> {
        let header_value = self
            .headers()
            .get("refresh_token")
            .ok_or(AuthError::MissingAuth("refresh_token".into()))?
            .to_str()
            .map_err(|_| AuthError::Unauthorized)?;

        Ok(header_value.into())
    }

    fn refresh_token_from_cookie(&self) -> Result<String, AuthError> {
        let refresh_token = self
            .cookie("refresh_token")
            .ok_or(AuthError::MissingAuth("refresh_token".into()))?
            .to_string();

        let refresh_token = refresh_token
            .strip_prefix("refresh_token")
            .ok_or(AuthError::InvalidTokenScheme)?;

        Ok(refresh_token.to_string())
    }
}
