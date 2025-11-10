use crate::base::{create_invalid_user, create_underage_user};
use proptest::prelude::*;

use crate::base::spawn_app;
use sqlx::Row;
use thalia::user::models::AccessRole;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[actix_web::test]
async fn staff_signup_returns_200_for_valid_data() {
    // Arrange
    let mut app = spawn_app().await;

    let staff_body = app.staff_to_json();

    // Mock server
    Mock::given(path("v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.get_mail_state().email_server)
        .await;

    // Act
    let response = app.post_staff_signup(&staff_body).await;
    assert_eq!(response.status().as_u16(), 200);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn customer_signup_returns_200_for_valid_data() {
    // Arrange
    let mut app = spawn_app().await;

    let customer_body = app.customer_to_json();

    // Mock server
    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.get_mail_state().email_server)
        .await;

    // Act
    let response = app.post_customer_signup(&customer_body).await;
    assert_eq!(response.status().as_u16(), 200);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn staff_signup_persists_the_new_profile() {
    // Arrange
    let mut app = spawn_app().await;

    // Mock server
    Mock::given(path("v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.get_mail_state().email_server)
        .await;

    let staff_body = app.staff_to_json();

    app.post_staff_signup(&staff_body).await;

    let saved = sqlx::query("SELECT email, access_role, is_confirmed FROM tuser")
        .fetch_one(&app.get_db_state().pg_pool)
        .await
        .expect("Failed to fetch staff profile");

    assert_eq!(
        saved.get::<String, _>("email"),
        app.get_test_users().get_staff().get_email().as_ref()
    );

    assert_eq!(
        saved.get::<AccessRole, _>("access_role"),
        *app.get_test_users().get_staff().get_access_role()
    );

    assert_eq!(
        saved.get::<bool, _>("is_confirmed"),
        *app.get_test_users().get_staff().get_is_confirmed()
    );

    app.clear_test_db().await;
}

#[actix_web::test]
async fn customer_signup_persists_the_new_profile() {
    // Arrange
    let mut app = spawn_app().await;

    // Mock server
    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.get_mail_state().email_server)
        .await;

    let customer_body = app.customer_to_json();

    app.post_customer_signup(&customer_body).await;

    let saved = sqlx::query("SELECT email, access_role, is_confirmed FROM tuser")
        .fetch_one(&app.get_db_state().pg_pool)
        .await
        .expect("Failed to fetch staff profile");

    assert_eq!(
        saved.get::<String, _>("email"),
        app.get_test_users().get_customer().get_email().as_ref()
    );

    assert_eq!(
        saved.get::<AccessRole, _>("access_role"),
        *app.get_test_users().get_customer().get_access_role()
    );

    assert_eq!(
        saved.get::<bool, _>("is_confirmed"),
        *app.get_test_users().get_customer().get_is_confirmed()
    );

    app.clear_test_db().await;
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn staff_signup_returns_400_for_any_invalid_input(body in create_invalid_user("staff")) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut app =spawn_app().await;

            // Mock server
            Mock::given(path("v3/send"))
                .and(method("POST"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&app.get_mail_state().email_server)
                .await;

            let response = app.post_staff_signup(&body).await;

            assert_eq!(response.status().as_u16(), 400, "Expected 400 for invalid profile: {:?}", body);


            app.clear_test_db().await;

        })
    }

    #[test]
    fn underage_staff_signup_returns_422(body in create_underage_user("staff")) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut app =spawn_app().await;

            // Mock server
            Mock::given(path("v3/send"))
                .and(method("POST"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&app.get_mail_state().email_server)
                .await;

            let response = app.post_staff_signup(&body).await;

            assert_eq!(response.status().as_u16(), 422, "Expected 422 for underage profile: {:?}", body);


            app.clear_test_db().await;

        })
    }

    #[test]
    fn underage_customer_signup_returns_422_for_any_invalid_input(body in create_underage_user("superuser")) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut app =spawn_app().await;

            // Mock server
            Mock::given(path("v3/send"))
                .and(method("POST"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&app.get_mail_state().email_server)
                .await;

            let response = app.post_customer_signup(&body).await;

            assert_eq!(response.status().as_u16(), 422, "Expected 422 for underage profile: {:?}", body);


            app.clear_test_db().await;

        })
    }

        #[test]
    fn customer_signup_returns_400_for_any_invalid_input(body in create_invalid_user("superuser")) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut app =spawn_app().await;

            // Mock server
            Mock::given(path("v3/send"))
                .and(method("POST"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&app.get_mail_state().email_server)
                .await;

            let response = app.post_customer_signup(&body).await;

            assert_eq!(response.status().as_u16(), 400, "Expected 400 for invalid profile: {:?}", body);


            app.clear_test_db().await;

        })
    }
}

#[actix_web::test]
async fn staff_sign_sends_confirmation_email_for_valid_data() {
    // Arrange
    let mut app = spawn_app().await;

    let staff_body = app.staff_to_json();

    // Mock server
    Mock::given(path("v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.get_mail_state().email_server)
        .await;

    app.post_staff_signup(&staff_body).await;

    app.clear_test_db().await;
}
