use crate::{
    authentication::{
        read_request_access_token,
        repo::{db_get_password_by_email, db_get_password_by_username},
        schemas::{
            AccessLevel, ActivateClaims, Credentials, LoginIdentifier, SessionMetadata, TokenClaims,
        },
        util::to_unix_expiry,
    },
    base::error::BaseError,
    telemetry::spawn_blocking_with_tracing,
};
use actix_web::dev::ServiceRequest;
use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core},
};
use error_stack::{Report, ResultExt};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sqlx::PgPool;
use uuid::Uuid;

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
) -> Result<(Uuid, String), Report<BaseError>> {
    let mut user_id: Option<Uuid> = None;
    let mut user_name = String::from("1");
    // Used to limit timing attack
    let mut expected_password = credentials.get_default_password().clone();

    if let Some((retrieved_user_id, username, retrieved_password)) =
        match credentials.get_login_identifier() {
            LoginIdentifier::Email(email) => {
                let (retrieved_user_id, username, retrieved_password) =
                    db_get_password_by_email(pool, email).await.change_context(
                        BaseError::InvalidCredentials {
                            message: "Invalid password or username".into(),
                        },
                    )?;

                Some((retrieved_user_id, username, retrieved_password))
            }
            LoginIdentifier::Username(username) => {
                let (retrieved_user_id, username, retrieved_password) =
                    db_get_password_by_username(pool, username)
                        .await
                        .change_context(BaseError::InvalidCredentials {
                            message: "Invalid password or username".into(),
                        })?;

                Some((retrieved_user_id, username, retrieved_password))
            }
        }
    {
        user_id = Some(retrieved_user_id);
        // Replace password with retrieved password
        expected_password = retrieved_password;

        user_name = username;
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

    if let Some(id) = user_id {
        return Ok((id, user_name));
    }

    Err(Report::new(BaseError::InvalidCredentials {
        message: "Wrong password or username".into(),
    }))
}

#[tracing::instrument("Create activate token", skip(user_id, email, expiry, secret_key))]
pub fn create_activate_token(
    user_id: Uuid,
    email: &str,
    expiry: u64,
    secret_key: &str,
    role: AccessLevel,
) -> Result<String, Report<BaseError>> {
    let expiry = to_unix_expiry(expiry)?;
    let claims = ActivateClaims::new(email.into(), user_id, expiry, role);

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .map_err(|e| BaseError::from(e.into_kind()))?;
    Ok(token)
}

#[tracing::instrument("Validate activate token", skip(token, secret_key))]
pub fn validate_activate_token(
    token: &str,
    secret_key: &str,
) -> Result<(String, Uuid, AccessLevel), BaseError> {
    let token_data = decode::<ActivateClaims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    // Converting wrapper error to kind
    .map_err(|e| e.into_kind())?;

    Ok((
        token_data.claims.get_sub(),
        token_data.claims.get_aud(),
        token_data.claims.get_role(),
    ))
}

#[tracing::instrument("Create token", skip(user_id, role, expiry, secret_key))]
pub fn create_token(
    user_id: Uuid,
    username: &str,
    role: AccessLevel,
    expiry: u64,
    secret_key: &str,
) -> Result<String, BaseError> {
    let claims = TokenClaims::new(user_id, expiry, username, role);

    let jwt = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .map_err(|e| e.into_kind())?;

    Ok(jwt)
}

#[tracing::instrument("Validate access token", skip(req, secret_key))]
pub fn validate_access_token(
    req: &ServiceRequest,
    secret_key: &str,
) -> Result<SessionMetadata, BaseError> {
    let access_token = read_request_access_token(req.headers())?;

    let access_token_data = decode::<TokenClaims>(
        &access_token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| e.into_kind())?;

    Ok(access_token_data.claims.into())
}

#[tracing::instrument("Validate refresh token", skip(refresh_token, secret_key))]
pub fn validate_refresh_token(
    refresh_token: &str,
    secret_key: &str,
) -> Result<SessionMetadata, BaseError> {
    let refresh_token_data = decode::<TokenClaims>(
        refresh_token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| e.into_kind())?;

    Ok(refresh_token_data.claims.into())
}
