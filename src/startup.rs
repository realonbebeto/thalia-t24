use crate::admin::routes::{
    create_account_type, create_chart_account, create_customer_account, staff_signup,
};
use crate::config::{Config, DatabaseConfig};
use crate::customer::routes::{activate_profile, customer_signup, open_customer_account};
use crate::index::{health_check, index_page};
use crate::ledger::routes::{get_journal_entry, get_journal_entry_by_id};
use crate::openapi_docs::ApiDoc;
use crate::transaction::routes::{deposit_funds, withdraw_funds};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn get_pgconnect_pool(config: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}

async fn run(listener: TcpListener) -> Result<Server, anyhow::Error> {
    let server = HttpServer::new(|| {
        let logger = TracingLogger::default();
        let openapi = ApiDoc::openapi();

        App::new()
            .wrap(logger)
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
            .service(
                web::scope("/home")
                    .route("/health", web::get().to(health_check))
                    .route("/index", web::get().to(index_page)),
            )
            .service(
                web::scope("/admin")
                    .route("/staff/signup", web::post().to(staff_signup))
                    .route("/user/signup", web::post().to(create_customer_account))
                    .route("/coa", web::post().to(create_chart_account))
                    .route("/coa", web::put().to(create_account_type)),
            )
            .service(
                web::scope("/ledger")
                    .route("/journal", web::get().to(get_journal_entry))
                    .route(
                        "/journal/{journal_id}",
                        web::get().to(get_journal_entry_by_id),
                    ),
            )
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
        let _pool = get_pgconnect_pool(&config.database);

        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let server = run(listener).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
