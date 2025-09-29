use crate::admin::routes::{
    create_account_type, create_chart_account, create_customer_account, staff_login, staff_signup,
};
use crate::authentication::schemas::{DefaultPassword, SecretKey};
use crate::base::schemas::AppBaseUri;
use crate::config::{Config, DatabaseConfig, Expiration};
use crate::customer::routes::{
    activate_profile, customer_login, customer_signup, open_customer_account,
};
use crate::index::{health_check, index_page};
use crate::ledger::routes::{get_journal_entry, get_journal_entry_by_id};
use crate::openapi_docs::ApiDoc;
use crate::transaction::routes::{deposit_funds, withdraw_funds};
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::{App, HttpServer, cookie::Key, dev::Server, web};
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn get_pgconnect_pool(config: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}

async fn run(
    listener: TcpListener,
    pgpool: PgPool,
    base_uri: &str,
    secret: &str,
    redis_uri: &str,
    expiration: Expiration,
    default_password: &str,
) -> Result<Server, anyhow::Error> {
    let pgpool = web::Data::new(pgpool);
    let base_uri = web::Data::new(AppBaseUri(base_uri.to_string()));
    let secret_key = Key::from(secret.as_bytes());
    let cookie_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(cookie_store).build();
    let redis_store = RedisSessionStore::new(redis_uri).await?;

    let secret = web::Data::new(SecretKey(secret.to_string()));
    let expiration = web::Data::new(expiration);
    let default_password = web::Data::new(DefaultPassword(default_password.to_string()));

    let server = HttpServer::new(move || {
        let logger = TracingLogger::default();
        let openapi = ApiDoc::openapi();

        App::new()
            .wrap(logger)
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .app_data(pgpool.clone())
            .app_data(base_uri.clone())
            .app_data(secret.clone())
            .app_data(expiration.clone())
            .app_data(default_password.clone())
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
            .service(
                web::scope("/home")
                    .route("/health", web::get().to(health_check))
                    .route("/index", web::get().to(index_page)),
            )
            .route("/staff/login", web::post().to(staff_login))
            .service(
                web::scope("/admin")
                    .route("/staff/signup", web::post().to(staff_signup))
                    .route("/user/signup", web::post().to(create_customer_account))
                    .route("/coa", web::post().to(create_chart_account))
                    .route("/type", web::post().to(create_account_type)),
            )
            .service(
                web::scope("/ledger")
                    .route("/journal", web::get().to(get_journal_entry))
                    .route(
                        "/journal/{journal_id}",
                        web::get().to(get_journal_entry_by_id),
                    ),
            )
            .route("/customer/login", web::post().to(customer_login))
            .service(
                web::scope("/customer")
                    .route("/signup", web::post().to(customer_signup))
                    .route("/activate/{token}", web::get().to(activate_profile))
                    .route("/account", web::post().to(open_customer_account)),
            )
            .service(
                web::scope("/transaction")
                    .route("/deposit", web::post().to(deposit_funds))
                    .route("/withdraw", web::post().to(withdraw_funds)),
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: &Config) -> Result<Self, anyhow::Error> {
        let pgpool = get_pgconnect_pool(&config.database);
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let server = run(
            listener,
            pgpool,
            &config.application.app_uri,
            &config.application.secret_key,
            &config.redis_uri,
            config.expiration.clone(),
            &config.application.default_password,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
