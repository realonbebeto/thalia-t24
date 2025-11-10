use anyhow::Context;
use deadpool_redis::{
    Config, Pool, Runtime,
    redis::{AsyncCommands, cmd},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct RedisPool {
    pool: Pool,
}

impl RedisPool {
    pub fn new(redis_uri: String) -> Result<Self, anyhow::Error> {
        let cfg = Config::from_url(redis_uri);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .context("Failed to create redis connection pool")?;

        Ok(Self { pool })
    }

    pub async fn set_token(&self, key: &str, value: Uuid, ttl: u64) -> Result<(), anyhow::Error> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get redis connection")?;

        let _: String = conn
            .set_ex(key, value.to_string(), ttl)
            .await
            .context("Failed to set refresh token in redis")?;

        Ok(())
    }

    pub async fn get_token(&self, key: &str) -> Result<Option<String>, anyhow::Error> {
        let mut conn = self.pool.get().await.unwrap();
        let value: Option<String> = cmd("GET")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .context("Failed to get refresh token from redis")?;

        Ok(value)
    }

    pub async fn remove_token(&self, key: &str) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await.unwrap();
        let _: u64 = conn
            .del(key)
            .await
            .context("Failed to delete refresh token from redis")?;

        Ok(())
    }
}
