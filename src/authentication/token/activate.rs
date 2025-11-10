use anyhow::Context;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::base::{Email, error::AuthError};
use crate::config::state::SecretKey;
use crate::infra::redis::RedisPool;
use crate::user::{models::AccessRole, schemas::User};

#[derive(Debug, serde::Deserialize, serde::Serialize, getset::CloneGetters)]
#[get_clone = "pub with_prefix"]
pub struct ActivateClaims {
    email: String,
    user_id: Uuid,
    exp: usize,
    role: AccessRole,
    token_use: String,
}

impl ActivateClaims {
    pub fn from(
        email: Email,
        user_id: Uuid,
        exp: usize,
        role: AccessRole,
        token_use: String,
    ) -> Self {
        Self {
            email: email.as_ref().to_string(),
            user_id,
            exp,
            role,
            token_use,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActivateHandler {
    secret: String,
    activate_ttl: usize,
}

impl ActivateHandler {
    pub fn new(secret: SecretKey, activate_ttl: u64) -> Self {
        Self {
            secret: secret.0,
            activate_ttl: activate_ttl as usize,
        }
    }

    pub async fn generate_activate_token(
        &self,
        user: &User,
        pool: &RedisPool,
    ) -> Result<String, anyhow::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get now as epoch")?
            .as_secs() as usize;

        let claims = user.to_activate_claims(now + self.activate_ttl);

        let activate_token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .context("Failed to encode activate token")?;

        pool.set_token(&activate_token.clone(), user.id, self.activate_ttl as u64)
            .await?;

        Ok(activate_token)
    }

    pub fn verify_activate_token(&self, token: &str) -> Result<ActivateClaims, anyhow::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get now as epoch")?
            .as_secs() as usize;
        let token_data = decode::<ActivateClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .context("Failed to decode activate token")?;

        if token_data.claims.token_use != "activate" || token_data.claims.exp < now {
            return Err(anyhow::anyhow!(AuthError::Unauthorized));
        }

        Ok(token_data.claims)
    }
}
