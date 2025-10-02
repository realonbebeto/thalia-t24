use crate::admin::routes::{
    confirm_staff, create_account_type, create_chart_account, create_customer_account, staff_login,
    staff_signup,
};
use crate::authentication::{
    middleware::{reject_unauthorized_customer, reject_unauthorized_staff},
    schemas::{DefaultPassword, SecretKey},
};
use crate::base::schemas::{AppBaseUri, AppRunParams};
use crate::config::{Config, DatabaseConfig};
use crate::customer::routes::{
    confirm_customer, customer_login, customer_signup, open_customer_account,
};
use crate::index::{health_check, index_page};
use crate::ledger::routes::{get_journal_entry, get_journal_entry_by_id};
use crate::openapi_docs::ApiDoc;
use crate::transaction::routes::{deposit_funds, withdraw_funds};
use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{App, HttpServer, cookie::Key, dev::Server, middleware::from_fn, web};
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn get_pgconnect_pool(config: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}

async fn run(app_params: AppRunParams<'_>) -> Result<Server, anyhow::Error> {
    let pgpool = web::Data::new(app_params.pgpool);
    let base_uri = web::Data::new(AppBaseUri(app_params.base_uri.to_string()));
    let secret_key = Key::from(app_params.secret.as_bytes());
    let cookie_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(cookie_store).build();
    let redis_store = RedisSessionStore::new(app_params.redis_uri).await?;

    let secret = web::Data::new(SecretKey(app_params.secret.to_string()));
    let expiration = web::Data::new(app_params.expiration);
    let default_password = web::Data::new(DefaultPassword(app_params.default_password.to_string()));
    let email_client = web::Data::new(app_params.email_client);

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
            .app_data(email_client.clone())
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
            .service(
                web::scope("/home")
                    .route("/health", web::get().to(health_check))
                    .route("/index", web::get().to(index_page)),
            )
            .route("/staff/signup", web::post().to(staff_signup))
            .route("/staff/login", web::post().to(staff_login))
            .route("/staff/confirm/{token}", web::get().to(confirm_staff))
            .service(
                web::scope("/staff")
                    .wrap(from_fn(reject_unauthorized_staff))
                    .route("/user/signup", web::post().to(create_customer_account))
                    .route("/coa", web::post().to(create_chart_account))
                    .route("/type", web::post().to(create_account_type)),
            )
            .service(
                web::scope("/ledger")
                    .wrap(from_fn(reject_unauthorized_staff))
                    .route("/journal", web::get().to(get_journal_entry))
                    .route(
                        "/journal/{journal_id}",
                        web::get().to(get_journal_entry_by_id),
                    ),
            )
            .route("/customer/signup", web::post().to(customer_signup))
            .route("/customer/login", web::post().to(customer_login))
            .route("/customer/confirm/{token}", web::get().to(confirm_customer))
            .service(
                web::scope("/customer")
                    .wrap(from_fn(reject_unauthorized_customer))
                    .route("/account", web::post().to(open_customer_account)),
            )
            .service(
                web::scope("/transaction")
                    .wrap(from_fn(reject_unauthorized_customer))
                    .route("/deposit", web::post().to(deposit_funds))
                    .route("/withdraw", web::post().to(withdraw_funds)),
            )
    })
    .listen(app_params.listener)?
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
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let email_client = config.email_client.client();

        let app_params = AppRunParams {
            listener,
            pgpool,
            base_uri: &config.application.app_uri,
            secret: &config.application.secret_key,
            redis_uri: &config.redis_uri,
            expiration: config.expiration.clone(),
            default_password: &config.application.default_password,
            email_client,
        };

        let server = run(app_params).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
