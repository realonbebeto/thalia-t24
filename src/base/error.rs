use crate::base::StdResponse;
use actix_web::HttpResponse;
use error_stack::Report;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Email is not a valid")]
    InvalidEmail,
    #[error("Username is not a valid")]
    InvalidUsername,
    #[error("Name is not valid")]
    InvalidName,
}

#[derive(Debug, thiserror::Error, Clone)]
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

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::AlreadyExists {
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
        };

        HttpResponse::build(status).json(StdResponse { message })
    }
}

// pub fn internal<E: std::fmt::Display>(err: E) -> AdminError {
//     AdminError::internal()
//     // ).attach(format!("Internal error: {}", err))
// }

// pub fn bad_request<E: std::fmt::Display>(err: E, context: &str) -> AdminError {
//     AdminError::bad_request(context)
//     // ).attach(format!("Bad request - {}: {}", context, err))
// }

// pub fn not_found<E: std::fmt::Display>(err: E, context: &str) -> AdminError {
//     AdminError::not_found(context)
//     // ).attach(format!("Not found - {}: {}", context, err))
// }

// pub fn service_unavailable<E: std::fmt::Display>(err: E) -> AdminError {
//     AdminError::service_unavailable()
//     // ).attach(format!("Service unavailable: {}", err))
// }

type Result<T> = std::result::Result<T, BaseError>;

// Important to convert Report<AdminError> to AppError automatically
impl From<Report<BaseError>> for BaseError {
    fn from(value: Report<BaseError>) -> Self {
        value.current_context().clone()
    }
}

pub trait ErrorExt<T> {
    fn to_internal(self) -> Result<T>;
    fn to_badrequest(self) -> Result<T>;
    fn to_notfound(self) -> Result<T>;
    fn to_serviceunavailable(self) -> Result<T>;
    fn to_alreadyexists(self) -> Result<T>;
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
}

#[derive(Debug, thiserror::Error)]
pub enum DBError {
    #[error("Database Fault: {message}")]
    DBFault { message: String },
    #[error("Record(s) not found")]
    NotFound,
}
