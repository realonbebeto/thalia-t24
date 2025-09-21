use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ActivateClaims {
    sub: String,
    iat: usize,
    aud: Uuid,
    exp: usize,
}

pub fn create_activate_token(
    user_id: Uuid,
    email: &str,
    expiry: u64,
    secret_key: &str,
) -> Result<String, anyhow::Error> {
    let now = SystemTime::now();
    let exp = (now + Duration::from_secs(expiry * 60))
        .duration_since(UNIX_EPOCH)?
        .as_secs() as usize;

    let iat = now.duration_since(UNIX_EPOCH)?.as_secs() as usize;

    let claims = ActivateClaims {
        sub: email.into(),
        aud: user_id,
        iat,
        exp,
    };

    let token = encode(
        &Header::new(Algorithm::HS512),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )?;
    Ok(token)
}

pub fn validate_activate_token(
    token: &str,
    secret_key: &str,
) -> Result<(String, Uuid), anyhow::Error> {
    let token_data = decode::<ActivateClaims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::new(Algorithm::HS512),
    )?;

    Ok((token_data.claims.sub, token_data.claims.aud))
}
