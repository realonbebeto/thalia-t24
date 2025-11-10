use sqlx::PgPool;

use crate::authentication::token::{ActivateHandler, TokenHandler};
use crate::infra::{aws::S3Client, redis::RedisPool};
use crate::notification::email_client::EmailClient;

#[derive(Debug, Clone)]
pub struct SecretKey(pub String);

#[derive(Debug, Clone)]
pub struct DefaultPassword(pub String);

#[derive(Debug)]
pub struct AppBaseUri(pub String);

#[derive(Debug)]
pub struct RedisUri(pub String);

#[derive(Debug)]
pub struct AppState {
    pub pgpool: PgPool,
    pub base_uri: AppBaseUri,
    pub secret: SecretKey,
    pub redis_uri: RedisUri,
    pub default_password: DefaultPassword,
    pub email_client: EmailClient,
    pub token_handler: TokenHandler,
    pub redis_pool: RedisPool,
    pub activate_handler: ActivateHandler,
    pub s3_client: S3Client,
}
