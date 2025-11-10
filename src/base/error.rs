use crate::base::StdResponse;
use actix_web::HttpResponse;

pub fn error_chain_format(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by: \n\t{}", cause)?;
        current = cause.source();
    }

    Ok(())
}

// For data format and constraint validation
#[derive(Debug, thiserror::Error, Clone)]
pub enum ValidationError {
    #[error("Invalid format for {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Missing required field: {0}")]
    UnrequiredField(String),

    #[error("Field too short: {field} (minimum: {min})")]
    TooShort { field: String, min: usize },

    #[error("Field too long: {field} (maximum: {max})")]
    TooLong { field: String, max: usize },

    #[error("Value out of range for {field} (must be between {min} and {max})")]
    OutOfRange {
        field: String,
        min: String,
        max: String,
    },
    #[error("Fields do not match: {0}")]
    Mismatch(String),

    #[error("Invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Too many {field}: expected {expected}, got {actual}")]
    TooMany {
        field: String,
        expected: usize,
        actual: usize,
    },

    #[error("Too few {field}: expected {expected}, got {actual}")]
    TooFew {
        field: String,
        expected: usize,
        actual: usize,
    },

    #[error("Invalid count for {field}: expected exactly {expected}, got {actual}")]
    InvalidCount {
        field: String,
        expected: usize,
        actual: usize,
    },
}

impl actix_web::ResponseError for ValidationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(StdResponse::from(&self.to_string()))
    }
}

// For authorization and permissions
#[derive(Debug, thiserror::Error, Clone)]
pub enum AuthError {
    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Expired: {0}")]
    Expired(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Missing authentication: {0}")]
    MissingAuth(String),

    #[error("Invalid token scheme")]
    InvalidTokenScheme,
}

impl actix_web::ResponseError for AuthError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AuthError::InvalidCredentials(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AuthError::InsufficientPermissions => actix_web::http::StatusCode::FORBIDDEN,
            AuthError::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            AuthError::Expired(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AuthError::MissingAuth(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AuthError::InvalidTokenScheme => actix_web::http::StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(StdResponse::from(&self.to_string()))
    }
}

// For business logic and state errors
#[derive(Debug, thiserror::Error, Clone)]
pub enum DomainError {
    #[error("Duplicate entry: {0}")]
    Duplicate(String),

    #[error("Entry not found: {0}")]
    NotFound(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl actix_web::ResponseError for DomainError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            DomainError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            DomainError::Duplicate(_) => actix_web::http::StatusCode::CONFLICT,
            DomainError::ConstraintViolation(_) => {
                actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
            }
            DomainError::InvalidState(_) => actix_web::http::StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(StdResponse::from(&self.to_string()))
    }
}

// Top-level error that encompasses all
#[derive(thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Internal(anyhow::Error),
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_format(self, f)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        if let Some(e) = value.downcast_ref::<ValidationError>() {
            return AppError::Validation(e.clone());
        }

        if let Some(e) = value.downcast_ref::<AuthError>() {
            return AppError::Auth(e.clone());
        }

        if let Some(e) = value.downcast_ref::<DomainError>() {
            return AppError::Domain(e.clone());
        }

        AppError::Internal(value)
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::Auth(e) => e.status_code(),
            AppError::Domain(e) => e.status_code(),
            AppError::Validation(e) => e.status_code(),
            AppError::Internal(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            AppError::Auth(e) => e.error_response(),
            AppError::Domain(e) => e.error_response(),
            AppError::Validation(e) => e.error_response(),
            AppError::Internal(_) => HttpResponse::build(self.status_code())
                .json(StdResponse::from("Internal server error")),
        }
    }
}

// Special sqlx error handling
pub trait SqlErrorExt<T> {
    fn to_app_err(self, context: &str) -> Result<T, AppError>;
}

impl<T> SqlErrorExt<T> for std::result::Result<T, sqlx::Error> {
    fn to_app_err(self, context: &str) -> Result<T, AppError> {
        let context: String = context.into();
        self.map_err(|e| match e {
            sqlx::Error::Database(v) => match v.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => {
                    AppError::Domain(DomainError::NotFound(context))
                }

                sqlx::error::ErrorKind::UniqueViolation => {
                    AppError::Domain(DomainError::Duplicate(context))
                }

                _ => AppError::Internal(anyhow::anyhow!(v).context(context)),
            },

            other => AppError::Internal(anyhow::anyhow!(other).context(context)),
        })
    }
}
