use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{App, HttpServer, cookie::Key, dev::Server, middleware::from_fn, web};
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::account::routes::open_customer_account;
use crate::authentication::middleware::{reject_unauthorized_customer, reject_unauthorized_staff};
use crate::config::{runtime::Config, state::AppState};
use crate::customer::routes::{confirm_customer, customer_login, customer_signup};
use crate::index::{health_check, index_page};
use crate::ledger::routes::{journal_entry, journal_entry_by_id};
use crate::openapi_docs::ApiDoc;
use crate::staff::routes::{
    confirm_staff, create_account_type, create_chart_account, create_customer_account, staff_login,
    staff_signup,
};
use crate::transaction::routes::{deposit_funds, withdraw_funds};

async fn run(listener: TcpListener, app_state: AppState) -> Result<Server, anyhow::Error> {
    let secret_key = Key::from(app_state.secret.0.as_bytes());
    let cookie_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(cookie_store).build();
    let redis_store = RedisSessionStore::new(app_state.redis_uri.0.clone()).await?;

    let app_state = web::Data::new(app_state);

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
            .app_data(app_state.clone())
            .service(Scalar::with_url("/docs", openapi))
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
                    .route("/type", web::post().to(create_account_type))
                    .route("/account", web::post().to(open_customer_account)),
            )
            .service(
                web::scope("/ledger")
                    .wrap(from_fn(reject_unauthorized_staff))
                    .route("/journal", web::get().to(journal_entry))
                    .route("/journal/{journal_id}", web::get().to(journal_entry_by_id)),
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
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();

        let app_state: AppState = config.try_into_state().await?;

        let server = run(listener, app_state).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
