use actix_web::{HttpRequest, dev::ServiceRequest};
use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core},
};
use error_stack::{Report, ResultExt};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sqlx::PgPool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    authentication::{
        read_request_access_token,
        repo::{db_get_password_by_email, db_get_password_by_username},
        schemas::{ActivateClaims, Credentials, LoginIdentifier, TokenClaims},
    },
    base::error::BaseError,
    telemetry::spawn_blocking_with_tracing,
};

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Failed to hash password")]
    HashError,
    #[error("Failed to build hashing params")]
    ParamError,
    #[error("Failed to build PHC string")]
    ParseError,
    #[error("Wrong password")]
    BadPassword,
}

#[tracing::instrument("Encode password", skip(password))]
pub fn encode_password(password: String) -> Result<String, Report<PasswordError>> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let password = Argon2::new(
        argon2::Algorithm::Argon2id,
        Version::V0x13,
        Params::new(27000, 2, 1, None).change_context(PasswordError::ParamError)?,
    )
    .hash_password(password.as_bytes(), &salt)
    .change_context(PasswordError::HashError)?
    .to_string();

    Ok(password)
}

#[tracing::instrument(name = "verify password", skip(expected_password, password))]
fn verify_password(expected_password: &str, password: &str) -> Result<(), Report<PasswordError>> {
    let expected_password = PasswordHash::new(expected_password)
        .change_context(PasswordError::ParseError)
        .attach("Failed to parse hash to PHC string format")?;

    Argon2::default()
        .verify_password(password.as_bytes(), &expected_password)
        .change_context(PasswordError::BadPassword)
        .attach("Invalid Password")?;

    Ok(())
}

#[tracing::instrument("Validate credentials", skip(pool, credentials))]
pub async fn validate_credentials(
    pool: &PgPool,
    credentials: Credentials,
) -> Result<Uuid, Report<BaseError>> {
    let mut user_id: Option<Uuid> = None;
    // Used to limit timing attack
    let mut expected_password = credentials.get_default_password().clone();

    if let Some((retrieved_user_id, retrieved_password)) = match credentials.get_login_identifier()
    {
        LoginIdentifier::Email(email) => {
            let (retrieved_user_id, retrieved_password) = db_get_password_by_email(pool, email)
                .await
                .change_context(BaseError::InvalidCredentials {
                    message: "Invalid password or username".into(),
                })?;

            Some((retrieved_user_id, retrieved_password))
        }
        LoginIdentifier::Username(username) => {
            let (retrieved_user_id, retrieved_password) =
                db_get_password_by_username(pool, username)
                    .await
                    .change_context(BaseError::InvalidCredentials {
                        message: "Invalid password or username".into(),
                    })?;

            Some((retrieved_user_id, retrieved_password))
        }
    } {
        user_id = Some(retrieved_user_id);
        // Replace password with retrieved password
        expected_password = retrieved_password;
    }

    if let Err(e) = spawn_blocking_with_tracing(move || {
        verify_password(&expected_password, credentials.get_password().as_ref())
    })
    .await
    .unwrap()
    {
        match e.current_context() {
            PasswordError::BadPassword | PasswordError::ParseError => {
                return Err(Report::new(BaseError::InvalidCredentials {
                    message: "Wrong password or username".into(),
                }));
            }

            _ => {
                return Err(Report::new(BaseError::Internal));
            }
        }
    }

    user_id.ok_or_else(|| {
        Report::new(BaseError::InvalidCredentials {
            message: "Wrong password or username".into(),
        })
    })
}

#[tracing::instrument("Create activate token")]
pub fn create_activate_token(
    user_id: Uuid,
    email: &str,
    expiry: u64,
    secret_key: &str,
) -> Result<String, Report<TokenError>> {
    let now = SystemTime::now();
    let exp = (now + Duration::from_secs(expiry * 60))
        .duration_since(UNIX_EPOCH)
        .change_context(TokenError::TimeError)?
        .as_secs() as usize;

    let iat = now
        .duration_since(UNIX_EPOCH)
        .change_context(TokenError::TimeError)?
        .as_secs() as usize;

    let claims = ActivateClaims::new(email.into(), iat, user_id, exp);

    let token = encode(
        &Header::new(Algorithm::HS512),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .change_context(TokenError::EncodeError)?;
    Ok(token)
}

#[tracing::instrument("Validate activate token")]
pub fn validate_activate_token(
    token: &str,
    secret_key: &str,
) -> Result<(String, Uuid), Report<TokenError>> {
    let token_data = decode::<ActivateClaims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS512),
    )
    .change_context(TokenError::DecodeError)?;

    Ok((token_data.claims.get_sub(), token_data.claims.get_aud()))
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Failed to create time from epoch")]
    Empty,
    #[error("Failed to create time from epoch")]
    TimeError,
    #[error("Failed to encode")]
    EncodeError,
    #[error("Failed to decode")]
    DecodeError,
}

#[tracing::instrument("Create token")]
pub fn create_token(
    user_id: Uuid,
    expiry: u64,
    secret_key: &str,
) -> Result<String, Report<TokenError>> {
    let exp = (SystemTime::now() + Duration::from_secs(expiry))
        .duration_since(UNIX_EPOCH)
        .change_context(TokenError::TimeError)?
        .as_secs() as usize;

    let claims = TokenClaims::new(user_id, exp);

    let jwt = encode(
        &Header::new(Algorithm::HS512),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .change_context(TokenError::EncodeError)?;

    Ok(jwt)
}

#[tracing::instrument("Validate access token")]
pub fn validate_access_token(
    req: &ServiceRequest,
    secret_key: &str,
) -> Result<Uuid, Report<TokenError>> {
    let access_token =
        read_request_access_token(req.headers()).change_context(TokenError::DecodeError)?;

    let access_token_data = decode::<TokenClaims>(
        &access_token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS512),
    )
    .change_context(TokenError::DecodeError)?;

    Ok(access_token_data.claims.get_sub())
}

#[tracing::instrument("Validate refresh token")]
pub fn validate_refresh_token(
    req: &HttpRequest,
    secret_key: &str,
) -> Result<Uuid, Report<TokenError>> {
    let refresh_token = req
        .cookie("refresh_token")
        .ok_or(TokenError::Empty)
        .change_context(TokenError::Empty)?
        .to_string();

    let refresh_token = refresh_token
        .strip_prefix("refresh_token")
        .ok_or(TokenError::Empty)
        .change_context(TokenError::Empty)?;

    let refresh_token_data = decode::<TokenClaims>(
        refresh_token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS512),
    )
    .change_context(TokenError::DecodeError)?;

    Ok(refresh_token_data.claims.get_sub())
}
