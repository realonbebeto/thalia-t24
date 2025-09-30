use envconfig::Envconfig;
use sqlx::ConnectOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;

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
pub struct Expiration {
    #[envconfig(from = "IDEMPOTENCY_EXPIRATION_SECS")]
    pub idempotency_expiration_secs: u64,
    #[envconfig(from = "ACCESS_TOKEN_EXPIRE_SECS")]
    pub access_token_expire_secs: u64,
    #[envconfig(from = "REFRESH_TOKEN_EXPIRE_SECS")]
    pub refresh_token_expire_secs: u64,
    #[envconfig(from = "ACTIVATE_TOKEN_EXPIRE_SECS")]
    pub activate_token_expire_secs: u64,
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
    pub expiration: Expiration,
}

pub fn get_config() -> Result<Config, envconfig::Error> {
    // Init config reader
    let config = Config::init_from_env().expect("Failed to parse required app env variables");

    Ok(config)
}
