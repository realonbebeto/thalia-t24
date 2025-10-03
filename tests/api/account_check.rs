use crate::base::spawn_app;
use uuid::Uuid;
#[actix_web::test]
async fn valid_user_account_creation_by_logged_in_staff_returns_200() {
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    app.get_coas().store_coas(app.get_pg_pool()).await;
    app.get_account_classes()
        .store_account_classes(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_staff().get_username().as_ref(), 
                                                "email": app.get_test_users().get_staff().get_email().as_ref(), 
                                                "password": app.get_test_users().get_staff().get_password().as_ref()});
    app.post_staff_login(&login_body).await;

    let acc_body = serde_json::json!({ "user_id": app.get_test_users().get_customer().get_id(), 
                                            "branch_id": Uuid::now_v7(), 
                                            "account_id": Uuid::now_v7(), 
                                            "account_class": app.get_account_classes().get_checking().get_id(), 
    "country_code": 840});

    let response = app
        .get_api_client()
        .post(format!("{}/staff/account", app.get_address()))
        .json(&acc_body)
        .send()
        .await
        .expect("Failed to create user account");

    assert_eq!(response.status().as_u16(), 200);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn invalid_user_account_creation_by_logged_in_staff_returns_400() {
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    app.get_coas().store_coas(app.get_pg_pool()).await;
    app.get_account_classes()
        .store_account_classes(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_staff().get_username().as_ref(), 
                                                "email": app.get_test_users().get_staff().get_email().as_ref(), 
                                                "password": app.get_test_users().get_staff().get_password().as_ref()});
    app.post_staff_login(&login_body).await;

    let acc_body = serde_json::json!({ "user_id": app.get_test_users().get_customer().get_id(), 
                                            "branch_id": Uuid::now_v7(), 
                                            "account_id": Uuid::now_v7(), 
                                            "account_class": app.get_account_classes().get_checking().get_id(),
                                            // bad country code
                                            "country_code": 899999});

    let response = app
        .get_api_client()
        .post(format!("{}/staff/account", app.get_address()))
        .json(&acc_body)
        .send()
        .await
        .expect("Failed to create user account");

    assert_eq!(response.status().as_u16(), 400);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn valid_user_account_creation_by_logged_in_customer_returns_200() {
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    app.get_coas().store_coas(app.get_pg_pool()).await;
    app.get_account_classes()
        .store_account_classes(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_customer().get_username().as_ref(), 
                                                "email": app.get_test_users().get_customer().get_email().as_ref(), 
                                                "password": app.get_test_users().get_customer().get_password().as_ref()});
    app.post_customer_login(&login_body).await;

    let acc_body = serde_json::json!({ "user_id": app.get_test_users().get_customer().get_id(), 
                                            "branch_id": Uuid::now_v7(), 
                                            "account_id": Uuid::now_v7(), 
                                            "account_class": app.get_account_classes().get_checking().get_id(),
                                            // bad country code
                                            "country_code": 840});

    let response = app
        .get_api_client()
        .post(format!("{}/customer/account", app.get_address()))
        .json(&acc_body)
        .send()
        .await
        .expect("Failed to create user account");

    assert_eq!(response.status().as_u16(), 200);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn invalid_user_account_creation_by_logged_in_customer_returns_400() {
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    app.get_coas().store_coas(app.get_pg_pool()).await;
    app.get_account_classes()
        .store_account_classes(app.get_pg_pool())
        .await;

    // Login
    let login_body = serde_json::json!({"username": app.get_test_users().get_customer().get_username().as_ref(), 
                                                "email": app.get_test_users().get_customer().get_email().as_ref(), 
                                                "password": app.get_test_users().get_customer().get_password().as_ref()});
    app.post_customer_login(&login_body).await;

    let acc_body = serde_json::json!({ "user_id": app.get_test_users().get_customer().get_id(), 
                                            "branch_id": Uuid::now_v7(), 
                                            "account_id": Uuid::now_v7(), 
                                            "account_class": app.get_account_classes().get_checking().get_id(),
                                            // bad country code
                                            "country_code": 89999});

    let response = app
        .get_api_client()
        .post(format!("{}/customer/account", app.get_address()))
        .json(&acc_body)
        .send()
        .await
        .expect("Failed to create user account");

    assert_eq!(response.status().as_u16(), 400);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn unauthenticated_staff_create_user_account_returns_401() {
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    app.get_coas().store_coas(app.get_pg_pool()).await;
    app.get_account_classes()
        .store_account_classes(app.get_pg_pool())
        .await;

    // No login
    let acc_body = serde_json::json!({ "user_id": app.get_test_users().get_customer().get_id(), 
                                            "branch_id": Uuid::now_v7(), 
                                            "account_id": Uuid::now_v7(), 
                                            "account_class": app.get_account_classes().get_checking().get_id(),
                                            // bad country code
                                            "country_code": 840});

    let response = app
        .get_api_client()
        .post(format!("{}/staff/account", app.get_address()))
        .json(&acc_body)
        .send()
        .await
        .expect("Failed to create user account");

    assert_eq!(response.status().as_u16(), 401);

    app.clear_test_db().await;
}

#[actix_web::test]
async fn unauthenticated_customer_create_user_account_returns_401() {
    let mut app = spawn_app().await;
    app.get_test_users()
        .store_test_users(app.get_pg_pool())
        .await;

    app.get_coas().store_coas(app.get_pg_pool()).await;
    app.get_account_classes()
        .store_account_classes(app.get_pg_pool())
        .await;

    // No Login
    let acc_body = serde_json::json!({ "user_id": app.get_test_users().get_customer().get_id(), 
                                            "branch_id": Uuid::now_v7(), 
                                            "account_id": Uuid::now_v7(), 
                                            "account_class": app.get_account_classes().get_checking().get_id(),
                                            // bad country code
                                            "country_code": 840});

    let response = app
        .get_api_client()
        .post(format!("{}/customer/account", app.get_address()))
        .json(&acc_body)
        .send()
        .await
        .expect("Failed to create user account");

    assert_eq!(response.status().as_u16(), 401);

    app.clear_test_db().await;
}
