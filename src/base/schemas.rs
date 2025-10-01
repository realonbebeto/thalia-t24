use crate::config::Expiration;
use crate::notification::email_client::EmailClient;
use sqlx::PgPool;
use std::net::TcpListener;

#[derive(Debug)]
pub struct AppBaseUri(pub String);

#[derive(Debug)]
pub struct TokenExpiry(pub u64);

#[derive(Debug)]
pub struct AppRunParams<'a> {
    pub listener: TcpListener,
    pub pgpool: PgPool,
    pub base_uri: &'a str,
    pub secret: &'a str,
    pub redis_uri: &'a str,
    pub expiration: Expiration,
    pub default_password: &'a str,
    pub email_client: EmailClient,
}
