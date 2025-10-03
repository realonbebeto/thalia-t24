use crate::base::spawn_app;

#[actix_web::test]
async fn valid_coa_creation_by_logged_in_staff_returns_200() {
    // Arrange
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_staff().get_username().as_ref(), 
                                                "email": app.get_test_users().get_staff().get_email().as_ref(), 
                                                "password": app.get_test_users().get_staff().get_password().as_ref()});
    app.post_staff_login(&login_body).await;

    let coa_body = serde_json::json!({"name": "Cash in Vault", "code":"21022", "coa_type":"asset", "currency":"USD"});

    let response = app
        .get_api_client()
        .post(&format!("{}/staff/coa", app.get_address()))
        .json(&coa_body)
        .send()
        .await
        .expect("Failed to create coa");

    assert_eq!(response.status().as_u16(), 200);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn invalid_coa_creation_by_logged_in_staff_returns_400() {
    // Arrange
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_staff().get_username().as_ref(), 
    "email": app.get_test_users().get_staff().get_email().as_ref(), 
    "password": app.get_test_users().get_staff().get_password().as_ref()});
    app.post_staff_login(&login_body).await;

    // Missing code and bad coa_type
    let coa_body = serde_json::json!({"name": "Cash in Vault", "coa_type":"ass", "currency":"USD"});

    let response = app.post_coa_creation(&coa_body).await;

    assert_eq!(response.status().as_u16(), 400);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn unauthenticated_staff_create_coa_returns_401() {
    // Arrange
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    let coa_body = serde_json::json!({"name": "Cash in Vault", "code":"21022", "coa_type":"asset", "currency":"USD"});

    let response = app.post_coa_creation(&coa_body).await;

    assert_eq!(response.status().as_u16(), 401);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn test_non_staff_user_create_coa_returns_403() {
    // Arrange
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_customer().get_username().as_ref(), 
    "email": app.get_test_users().get_customer().get_email().as_ref(), 
    "password": app.get_test_users().get_customer().get_password().as_ref()});
    app.post_customer_login(&login_body).await;

    let coa_body = serde_json::json!({"name": "Cash in Vault", "code":"21022", "coa_type":"asset", "currency":"USD"});

    let response = app.post_coa_creation(&coa_body).await;

    assert_eq!(response.status().as_u16(), 403);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn test_create_coa_with_existing_identifier_returns_409() {
    // Arrange
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_staff().get_username().as_ref(), 
    "email": app.get_test_users().get_staff().get_email().as_ref(), 
    "password": app.get_test_users().get_staff().get_password().as_ref()});
    app.post_staff_login(&login_body).await;

    let coa_body = serde_json::json!({"name": "Cash in Vault", "code":"21022", "coa_type":"asset", "currency":"USD"});

    let response = app.post_coa_creation(&coa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_coa_creation(&coa_body).await;

    assert_eq!(response.status().as_u16(), 409);

    app.clear_test_db().await;
}
