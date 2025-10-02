use crate::base::StdResponse;
use crate::base::spawn_app;

#[actix_web::test]
async fn health_check_works() {
    // Arrange
    let mut app = spawn_app().await;

    // Act
    let response = app.get_health().await;

    // assert
    assert!(response.status().is_success());
    let response: StdResponse = response.json().await.unwrap();
    assert!(response.message.contains("Up and running"));

    app.clear_test_db().await;
}
