use envconfig::Envconfig;
use sqlx::ConnectOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;

use crate::base::Email;
use crate::infra::aws::S3Client;
use crate::notification::email_client::EmailClient;

#[derive(serde::Deserialize, Envconfig, Debug)]
pub struct DatabaseConfig {
    #[envconfig(from = "DB_USERNAME")]
    pub db_username: String,
    #[envconfig(from = "DB_PASSWORD")]
    pub db_password: String,
    #[envconfig(from = "DB_PORT", default = "5432")]
    pub db_port: u16,
    #[envconfig(from = "DB_HOST")]
    pub db_host: String,
    #[envconfig(from = "DB_NAME")]
    pub db_name: String,
    #[envconfig(from = "REQUIRE_SSL")]
    pub require_ssl: bool,
}

impl DatabaseConfig {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Allow
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.db_host)
            .username(&self.db_username)
            .password(&self.db_password)
            .port(self.db_port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.db_name)
            .log_statements(tracing_log::log::LevelFilter::Trace)
    }
}

#[derive(serde::Deserialize, Debug)]
pub enum Environment {
    Local,
    Production,
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

#[derive(serde::Deserialize, Envconfig, Debug)]
pub struct AppConfig {
    #[envconfig(from = "APP_PORT")]
    pub port: u16,
    #[envconfig(from = "APP_HOST")]
    pub host: String,
    #[envconfig(from = "APP_ENVIRONMENT")]
    pub app_env: Environment,
    #[envconfig(from = "APP_URI")]
    pub app_uri: String,
    #[envconfig(from = "SECRET_KEY")]
    pub secret_key: String,
    #[envconfig(from = "DEFAULT_PASSWORD")]
    pub default_password: String,
}

#[derive(serde::Deserialize, Envconfig, Debug, Clone)]
pub struct Ttl {
    #[envconfig(from = "IDEMPOTENCY_TTL_SECS")]
    pub idempotency_ttl_secs: u64,
    #[envconfig(from = "ACCESS_TTL_SECS")]
    pub access_ttl_secs: u64,
    #[envconfig(from = "REFRESH_TTL_SECS")]
    pub refresh_ttl_secs: u64,
    #[envconfig(from = "ACTIVATE_TTL_SECS")]
    pub activate_ttl_secs: u64,
    #[envconfig(from = "SESSION_TTL")]
    pub session_ttl: u64,
}

#[derive(serde::Deserialize, Envconfig, Debug)]
pub struct S3Settings {
    #[envconfig(from = "AWS_REGION")]
    pub region: String,
    #[envconfig(from = "IMAGE_BUCKET")]
    pub image_bucket: String,
}

impl S3Settings {
    pub async fn client(&self) -> S3Client {
        S3Client::default(self.region.clone(), self.image_bucket.clone()).await
    }
}

#[derive(serde::Deserialize, Envconfig, Debug)]
pub struct Config {
    #[envconfig(nested)]
    pub database: DatabaseConfig,
    #[envconfig(nested)]
    pub application: AppConfig,
    #[envconfig(from = "REDIS_URI")]
    pub redis_uri: String,
    #[envconfig(from = "ENV_FILTER")]
    pub env_filter: String,
    #[envconfig(from = "BUNYAN_FORMATTING_NAME")]
    pub bunyan_formatting_name: String,
    #[envconfig(nested)]
    pub ttl: Ttl,
    #[envconfig(nested)]
    pub email_client: EmailClientSettings,
    #[envconfig(nested)]
    pub s3_client: S3Settings,
}

pub fn get_config() -> Result<Config, anyhow::Error> {
    // Init config reader
    let config = Config::init_from_env()?;

    Ok(config)
}

#[derive(serde::Deserialize, Envconfig, Debug)]
pub struct EmailClientSettings {
    #[envconfig(from = "APP_PORT")]
    pub port: u16,
    #[envconfig(from = "APP_HOST")]
    pub host: String,
    #[envconfig(from = "EMAIL_BASE_URI")]
    pub email_base_uri: String,
    #[envconfig(from = "SENDER_EMAIL")]
    pub sender_email: String,
    #[envconfig(from = "PUBLIC_EMAIL_KEY")]
    pub public_email_key: String,
    #[envconfig(from = "PRIVATE_EMAIL_KEY")]
    pub private_email_key: String,
    #[envconfig(from = "TIMEOUT_MS")]
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<Email, anyhow::Error> {
        Email::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }

    pub fn client(&self) -> Result<EmailClient, anyhow::Error> {
        let sender_email = self.sender()?;
        let timeout = self.timeout();

        EmailClient::new(
            &self.email_base_uri,
            sender_email,
            &self.private_email_key,
            &self.public_email_key,
            timeout,
        )
    }
}
