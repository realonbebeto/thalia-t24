use actix_web::dev::ServiceRequest;
use anyhow::Context;
use derive_more::{Constructor, Deref};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    authentication::token::TokenExtractor,
    base::{
        Email,
        error::{AppError, AuthError},
    },
    config::state::SecretKey,
    infra::redis::RedisPool,
    user::models::AccessRole,
};

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Deref, Constructor,
)]
#[serde(transparent)]
pub struct JwtTtl(pub Duration);

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Deref, Constructor,
)]
pub struct RefreshTtl(pub Duration);

#[derive(
    Debug, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq, Hash, getset::Getters,
)]
#[get = "pub with_prefix"]
pub struct SessionClaims {
    sub: Uuid,
    user_id: Uuid,
    email: Email,
    auth_time: usize,
    iat: usize,
    exp: usize,
    role: AccessRole,
    jti: Uuid,
    token_use: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshToken {
    pub user_id: Uuid,
    pub sub: Uuid,
    pub email: Email,
    pub iat: usize,
    pub jti: Uuid,
    pub access_ttl: JwtTtl,
    pub exp: usize,
    pub refresh_ttl: RefreshTtl,
    pub role: AccessRole,
    pub auth_time: usize,
    pub revoked: bool,
    pub token_use: String,
}

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct TokenHandler {
    secret: String,
    access_ttl: usize,
    refresh_ttl: usize,
}

impl TokenHandler {
    pub fn new(secret: SecretKey, access_ttl: u64, refresh_ttl: u64) -> Self {
        Self {
            secret: secret.0,
            access_ttl: access_ttl as usize,
            refresh_ttl: refresh_ttl as usize,
        }
    }

    pub async fn generate_tokens(
        &self,
        pool: &RedisPool,
        user_id: Uuid,
        email: Email,
        role: AccessRole,
    ) -> Result<(TokenPair, SessionClaims), anyhow::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get now as epoch")?
            .as_secs() as usize;

        let jti = Uuid::now_v7();

        let access_claims = SessionClaims {
            sub: user_id,
            user_id,
            email: email.clone(),
            auth_time: now,
            iat: now,
            exp: now + self.access_ttl,
            role: role.clone(),
            jti,
            token_use: "access".into(),
        };

        let access_token = encode(
            &Header::new(Algorithm::HS256),
            &access_claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .context("Falied to encode access claims")?;

        let refresh_claims = RefreshToken {
            sub: user_id,
            user_id,
            email,
            role,
            iat: now,
            jti,
            access_ttl: JwtTtl(Duration::from_secs(self.access_ttl as u64)),
            exp: now + self.refresh_ttl,
            refresh_ttl: RefreshTtl(Duration::from_secs(self.refresh_ttl as u64)),
            auth_time: now,
            revoked: false,
            token_use: "refresh".into(),
        };

        let refresh_token = encode(
            &Header::new(Algorithm::HS256),
            &refresh_claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .context("Falied to encode refresh claims")?;

        pool.set_token(&refresh_token.clone(), jti, self.refresh_ttl as u64)
            .await?;

        Ok((
            TokenPair {
                access_token,
                refresh_token,
            },
            access_claims,
        ))
    }

    fn verify_access_token(&self, token: &str) -> Result<SessionClaims, anyhow::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to convert seconds expirty to unix time")?
            .as_secs() as usize;

        let token_data = decode::<SessionClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .context("Failed to decode access token")?;

        if token_data.claims.token_use != "access" || token_data.claims.exp <= now {
            return Err(anyhow::anyhow!(AuthError::Unauthorized));
        }

        Ok(token_data.claims)
    }

    pub fn verify_from_service_req(
        &self,
        req: &ServiceRequest,
    ) -> Result<SessionClaims, anyhow::Error> {
        let token = req.bearer_token()?;
        self.verify_access_token(&token)
    }

    pub async fn refresh_tokens(
        &self,
        pool: RedisPool,
        refresh_token: &str,
    ) -> Result<(TokenPair, SessionClaims), anyhow::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get now as epoch")?
            .as_secs() as usize;

        let token_data = decode::<RefreshToken>(
            refresh_token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .context("Failed to decode refresh token")?;

        if token_data.claims.token_use != "refresh" || token_data.claims.exp < now {
            return Err(anyhow::anyhow!(AppError::Auth(AuthError::Unauthorized)));
        }

        match pool.get_token(refresh_token).await? {
            Some(_) => {
                pool.remove_token(refresh_token).await?;
                let tokens = self
                    .generate_tokens(
                        &pool,
                        token_data.claims.user_id,
                        token_data.claims.email,
                        token_data.claims.role,
                    )
                    .await?;

                Ok(tokens)
            }
            None => Err(anyhow::anyhow!(AppError::Auth(AuthError::Unauthorized))),
        }
    }

    pub async fn revoke_refresh_token(
        &self,
        pool: RedisPool,
        refresh_token: &str,
    ) -> Result<(), anyhow::Error> {
        pool.remove_token(refresh_token).await?;

        Ok(())
    }
}
