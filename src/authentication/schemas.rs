use getset::{CloneGetters, Getters};
use uuid::Uuid;

use crate::base::{Email, Password, Username, error::ValidationError};
use error_stack::Report;

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
    iat: usize,
    aud: Uuid,
    exp: usize,
}

impl ActivateClaims {
    pub fn new(sub: String, iat: usize, aud: Uuid, exp: usize) -> Self {
        ActivateClaims { sub, iat, aud, exp }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, CloneGetters)]
#[get_clone = "pub with_prefix"]
pub struct TokenClaims {
    sub: Uuid,
    exp: usize,
}

impl TokenClaims {
    pub fn new(sub: Uuid, exp: usize) -> Self {
        TokenClaims { sub, exp }
    }
}
