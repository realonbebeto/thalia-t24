#[cfg(test)]
mod tests {
    use redis::Commands;
    use reqwest::cookie::Cookie;
    use uuid::Uuid;

    use crate::base::{StdResponse, spawn_app};
    #[actix_web::test]
    async fn error_flash_message_is_set_on_staff_login_failure() {
        // Setup
        let mut app = spawn_app().await;

        // Act 1 - Try Login with bad credentials
        let login_body = serde_json::json!({"username": "random-username", "email": "random-email@gmail.com", "password": "random-password"});
        let response = app.post_staff_login(&login_body).await;
        assert_eq!(response.status().as_u16(), 401);

        // Read headers
        let cookie = response
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();

        assert!(cookie.contains("Staff%20login%20unsuccessful"));

        let response_body: StdResponse = response.json().await.unwrap();

        assert!(
            response_body
                .message
                .contains("Invalid password or username")
        );

        app.clear_test_db().await;
    }

    #[actix_web::test]
    async fn successful_staff_login() {
        // Setup
        let mut app = spawn_app().await;
        app.get_test_users()
            .store_test_users(app.get_pg_pool())
            .await;

        // Act 1 - Login in
        let login_body = serde_json::json!({"username": app.get_test_users().get_staff().get_username().as_ref(), "email": app.get_test_users().get_staff().get_email().as_ref(), "password": app.get_test_users().get_staff().get_password().as_ref()});
        let response = app.post_staff_login(&login_body).await;

        assert_eq!(response.status().as_u16(), 200);

        app.clear_test_db().await;
    }
}
