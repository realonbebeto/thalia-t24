use crate::authentication::util::to_unix_expiry;
use crate::base::error::BaseError;
use crate::base::{Email, Password, Username, error::ValidationError};
use error_stack::Report;
use getset::{CloneGetters, Getters, Setters};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug)]
pub struct SecretKey(pub String);

#[derive(Debug)]
pub struct DefaultPassword(pub String);

#[derive(Debug)]
pub struct ActivateExpiryTime(pub u64);

#[derive(Debug)]
pub enum LoginIdentifier {
    Username(String),
    Email(String),
}

#[derive(Debug, Getters)]
#[get = "pub with_prefix"]
pub struct Credentials {
    login_identifier: LoginIdentifier,
    password: Password,
    // default password is passed as env variable used to limit timing attack
    default_password: String,
}

impl Credentials {
    pub fn from(
        login_data: LoginRequest,
        default_password: &str,
    ) -> Result<Self, Report<ValidationError>> {
        let login_identifier = match (login_data.email, login_data.username) {
            (Some(email), _) => {
                let email = Email::parse(email)?;
                LoginIdentifier::Email(email.as_ref().to_string())
            }
            (_, Some(username)) => {
                let username = Username::parse(username)?;
                LoginIdentifier::Username(username.as_ref().to_string())
            }
            (_, _) => {
                return Err(Report::new(ValidationError::MissingCredentials));
            }
        };

        Ok(Credentials {
            login_identifier,
            password: Password::parse(login_data.password)?,
            default_password: default_password.to_owned(),
        })
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}

impl LoginRequest {
    pub fn non_empty_email_username(&self) -> bool {
        self.email.is_none() && self.username.is_none()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, CloneGetters)]
#[get_clone = "pub with_prefix"]
pub struct ActivateClaims {
    sub: String,
    aud: Uuid,
    exp: usize,
    role: AccessLevel,
}

impl ActivateClaims {
    pub fn new(sub: String, aud: Uuid, exp: usize, role: AccessLevel) -> Self {
        ActivateClaims {
            sub,
            aud,
            exp,
            role,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, getset::Getters)]
#[get = "pub with_prefix"]
pub struct SessionMetadata {
    user_id: Uuid,
    exp: usize,
    iat: usize,
    auth_time: usize,
    username: String,
    persissions: AccessLevel,
}

impl SessionMetadata {
    pub fn new(
        user_id: Uuid,
        expiry: u64,
        username: &str,
        persissions: AccessLevel,
    ) -> Result<Self, Report<BaseError>> {
        let exp = to_unix_expiry(expiry)?;
        let iat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BaseError::Internal)?
            .as_secs() as usize;
        let auth_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BaseError::Internal)?
            .as_secs() as usize;
        Ok(Self {
            user_id,
            exp,
            iat,
            auth_time,
            username: username.into(),
            persissions,
        })
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, CloneGetters, Setters)]
#[get_clone = "pub with_prefix"]
pub struct TokenClaims {
    user_id: Uuid,
    exp: usize,
    iat: usize,
    auth_time: usize,
    username: String,
    persissions: AccessLevel,
}

impl TokenClaims {
    pub fn new(
        user_id: Uuid,
        expiry: u64,
        username: &str,
        persissions: AccessLevel,
    ) -> Result<Self, Report<BaseError>> {
        let exp = to_unix_expiry(expiry)?;
        let iat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BaseError::Internal)?
            .as_secs() as usize;

        let auth_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BaseError::Internal)?
            .as_secs() as usize;

        Ok(Self {
            user_id,
            exp,
            iat,
            auth_time,
            username: username.into(),
            persissions,
        })
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum AccessLevel {
    Manager,
    Superuser,
    Customer,
}

impl From<AccessLevel> for String {
    fn from(value: AccessLevel) -> Self {
        match value {
            AccessLevel::Customer => "customer".into(),
            AccessLevel::Manager => "manager".into(),
            AccessLevel::Superuser => "superuser".into(),
        }
    }
}

impl From<TokenClaims> for SessionMetadata {
    fn from(value: TokenClaims) -> Self {
        Self {
            user_id: value.get_user_id(),
            exp: value.get_exp(),
            iat: value.get_iat(),
            auth_time: value.get_auth_time(),
            username: value.get_username(),
            persissions: value.get_persissions(),
        }
    }
}
