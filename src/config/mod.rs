pub mod runtime;
pub mod state;

use crate::authentication::token::{ActivateHandler, TokenHandler};
use crate::infra::redis::RedisPool;
use runtime::{Config, DatabaseConfig};
use state::{AppBaseUri, AppState, DefaultPassword, RedisUri, SecretKey};

use sqlx::{PgPool, postgres::PgPoolOptions};

pub fn get_pgconnect_pool(config: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}

impl Config {
    pub async fn try_into_state(&self) -> Result<AppState, anyhow::Error> {
        let pgpool = get_pgconnect_pool(&self.database);
        let base_uri = AppBaseUri(self.application.app_uri.clone());
        let secret = SecretKey(self.application.secret_key.clone());
        let redis_uri = RedisUri(self.redis_uri.clone());
        let default_password = DefaultPassword(self.application.default_password.clone());
        let token_handler = TokenHandler::new(
            secret.clone(),
            self.ttl.access_ttl_secs,
            self.ttl.refresh_ttl_secs,
        );
        let redis_pool = RedisPool::new(self.redis_uri.clone())?;
        let activate_handler = ActivateHandler::new(secret.clone(), self.ttl.activate_ttl_secs);

        let s3_client = self.s3_client.client().await;

        Ok(AppState {
            pgpool,
            base_uri,
            secret,
            redis_uri,
            default_password,
            email_client: self.email_client.client()?,
            token_handler,
            redis_pool,
            activate_handler,
            s3_client,
        })
    }
}
