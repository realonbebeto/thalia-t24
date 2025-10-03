use crate::base::StdResponse;
use actix_web::HttpResponse;
use jsonwebtoken::errors::ErrorKind;

#[derive(Debug, thiserror::Error, Clone)]
pub enum ValidationError {
    #[error("Email is not valid")]
    InvalidEmail,
    #[error("Username is not valid")]
    InvalidUsername,
    #[error("Name is not valid")]
    InvalidName,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Missing credential(s)")]
    MissingCredentials,
    #[error("Missing credential(s)")]
    WrongAccessRole,
}

#[derive(Debug, thiserror::Error, Clone, serde::Serialize)]
pub enum BaseError {
    #[error("Bad request: {message}")]
    BadRequest { message: String },
    #[error("Not Found: {message}")]
    NotFound { message: String },
    #[error("Internal Server Error")]
    Internal,
    #[error("Service Unavailable")]
    ServiceUnavailable,
    #[error("Conflict: {message}")]
    AlreadyExists { message: String },
    #[error("Invalid Credentials: {message}")]
    InvalidCredentials { message: String },
    #[error("Access denied to resource")]
    Forbidden,
}

impl BaseError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    pub fn internal() -> Self {
        Self::Internal
    }

    pub fn service_unavailable() -> Self {
        Self::ServiceUnavailable
    }

    pub fn forbidden() -> Self {
        Self::Forbidden
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::AlreadyExists {
            message: message.into(),
        }
    }

    pub fn invalid_credentials(message: impl Into<String>) -> Self {
        Self::InvalidCredentials {
            message: message.into(),
        }
    }
}

impl actix_web::ResponseError for BaseError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let (status, message) = match self {
            BaseError::BadRequest { message } => {
                (actix_web::http::StatusCode::BAD_REQUEST, message)
            }
            BaseError::NotFound { message } => (actix_web::http::StatusCode::NOT_FOUND, message),
            BaseError::Internal => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                &String::from("Internal Server Error"),
            ),
            BaseError::ServiceUnavailable => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                &String::from("Service Unavailable"),
            ),
            BaseError::AlreadyExists { message } => {
                (actix_web::http::StatusCode::CONFLICT, message)
            }
            BaseError::InvalidCredentials { message } => {
                (actix_web::http::StatusCode::UNAUTHORIZED, message)
            }
            BaseError::Forbidden => (
                actix_web::http::StatusCode::FORBIDDEN,
                &String::from("Forbidden"),
            ),
        };

        HttpResponse::build(status).json(StdResponse::from(message))
    }
}

type Result<T> = std::result::Result<T, BaseError>;

pub trait ErrorExt<T> {
    fn to_internal(self) -> Result<T>;
    fn to_badrequest(self) -> Result<T>;
    fn to_notfound(self) -> Result<T>;
    fn to_serviceunavailable(self) -> Result<T>;
    fn to_alreadyexists(self) -> Result<T>;
    fn to_invalidcredentials(self) -> Result<T>;
    fn to_forbidden(self) -> Result<T>;
}

impl<T, E: std::fmt::Display> ErrorExt<T> for std::result::Result<T, E> {
    fn to_badrequest(self) -> Result<T> {
        self.map_err(|e| BaseError::bad_request(e.to_string()))
    }

    fn to_internal(self) -> Result<T> {
        self.map_err(|_| BaseError::internal())
    }

    fn to_notfound(self) -> Result<T> {
        self.map_err(|e| BaseError::not_found(e.to_string()))
    }
    fn to_serviceunavailable(self) -> Result<T> {
        self.map_err(|_| BaseError::service_unavailable())
    }

    fn to_alreadyexists(self) -> Result<T> {
        self.map_err(|e| BaseError::already_exists(e.to_string()))
    }

    fn to_invalidcredentials(self) -> Result<T> {
        self.map_err(|e| BaseError::invalid_credentials(e.to_string()))
    }

    fn to_forbidden(self) -> Result<T> {
        self.map_err(|_| BaseError::forbidden())
    }
}

impl From<ErrorKind> for BaseError {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::InvalidToken => Self::InvalidCredentials {
                message: "Authenticate before accessing this resource.".into(),
            },
            ErrorKind::MissingRequiredClaim(_) => Self::BadRequest {
                message: "Your token format is invalid.".into(),
            },
            ErrorKind::ExpiredSignature | ErrorKind::InvalidSignature => Self::InvalidCredentials {
                message: "You're not authorized".into(),
            },
            _ => Self::Internal,
        }
    }
}

// MainError
#[derive(Debug, thiserror::Error, Clone, serde::Serialize)]
pub enum MainError {
    #[error("{value}")]
    Runtime { value: String },
}
