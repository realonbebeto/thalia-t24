mod invalid_user;
mod test_user;
pub use invalid_user::create_invalid_user;

use crate::base::test_user::TestUsers;
use error_stack::ResultExt;
use getset::Getters;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use thalia::{
    config::{DatabaseConfig, get_config},
    notification::email_client::EmailClient,
    startup::{Application, get_pgconnect_pool},
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};
use uuid::Uuid;
use wiremock::MockServer;

#[derive(Debug, serde::Deserialize)]
pub struct StdResponse {
    pub message: String,
}

#[derive(Debug, Getters)]
#[get = "pub with_prefix"]
pub struct TestApp {
    address: String,
    pg_pool: PgPool,
    connection: PgConnection,
    db_name: String,
    email_server: MockServer,
    port: u16,
    test_users: TestUsers,
    api_client: reqwest::Client,
    test_idem_expiration: u64,
    email_client: EmailClient,
}

impl TestApp {
    fn new(
        address: String,
        pg_pool: PgPool,
        connection: PgConnection,
        db_name: String,
        email_server: MockServer,
        port: u16,
        test_users: TestUsers,
        api_client: reqwest::Client,
        test_idem_expiration: u64,
        email_client: EmailClient,
    ) -> Self {
        TestApp {
            address,
            pg_pool,
            connection,
            db_name,
            email_server,
            port,
            test_users,
            api_client,
            test_idem_expiration,
            email_client,
        }
    }

    pub async fn get_health(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/home/health", &self.address))
            .send()
            .await
            .expect("Failed to execute")
    }

    pub async fn post_customer_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/customer/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to login customer")
    }

    pub async fn post_customer_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/customer/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to login staff")
    }

    pub async fn post_staff_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/staff/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to login staff")
    }

    pub async fn post_staff_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/staff/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to login staff")
    }

    pub async fn post_coa_creation<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/staff/coa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to create coa")
    }

    pub async fn clear_test_db(&mut self) {
        sqlx::query(format!(r#"DROP DATABASE "{}" WITH (FORCE);"#, self.db_name).as_str())
            .execute(&mut self.connection)
            .await
            .expect("Failed to drop database");
    }

    pub fn staff_to_json(&self) -> serde_json::Value {
        let value = serde_json::json!({"first_name": self.get_test_users().get_staff().get_first_name().as_ref(),
                                            "last_name": self.get_test_users().get_staff().get_last_name().as_ref(),
                                            "date_of_birth":self.get_test_users().get_staff().get_date_of_birth(),
                                            "username":self.get_test_users().get_staff().get_username().as_ref(),
                                            "password":self.get_test_users().get_staff().get_password().as_ref(),
                                            "email":self.get_test_users().get_staff().get_email().as_ref(),
                                            "access_role":"superuser"});
        value
    }

    pub fn customer_to_json(&self) -> serde_json::Value {
        let value = serde_json::json!({"first_name": self.get_test_users().get_customer().get_first_name().as_ref(),
                                            "last_name": self.get_test_users().get_customer().get_last_name().as_ref(),
                                            "date_of_birth":self.get_test_users().get_customer().get_date_of_birth(),
                                            "username":self.get_test_users().get_customer().get_username().as_ref(),
                                            "password":self.get_test_users().get_customer().get_password().as_ref(),
                                            "email":self.get_test_users().get_customer().get_email().as_ref(),
                                            "access_role":"customer"});
        value
    }
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            get_tracing_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_tracing_subscriber(subscriber);
    } else {
        let subscriber =
            get_tracing_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_tracing_subscriber(subscriber);
    }
});

async fn configure_db(config: &DatabaseConfig) -> (PgPool, PgConnection) {
    //Connect to database server
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres via connection");

    // Create test database

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .attach("Failed to create database")
        .unwrap();

    // Run migrations
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .attach("Failed to connect to Postgres via pool")
        .unwrap();

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .attach("Failed to migrate the test database")
        .unwrap();
    (connection_pool, connection)
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let config = {
        let mut config = get_config().expect("Failed to load configuration");

        config.database.db_name = Uuid::now_v7().to_string();

        config.application.port = 0;

        config.email_client.email_base_uri = format!("{}/v3/send", email_server.uri());

        config
    };

    // Create and migrate test db
    let (_, connection) = configure_db(&config.database).await;

    let app = Application::build(&config)
        .await
        .expect("Failed to build application");

    let port = app.port();
    let address = format!("http://127.0.0.1:{}", port);
    let _ = tokio::spawn(app.run_until_stopped());
    let test_users = TestUsers::generate_users();

    let api_client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp::new(
        address,
        get_pgconnect_pool(&config.database),
        connection,
        config.database.db_name,
        email_server,
        port,
        test_users,
        api_client,
        config.expiration.idempotency_expiration_secs,
        config.email_client.client(),
    );

    test_app
}
