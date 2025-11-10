use crate::base::spawn_app;

async fn authenticated_staff_over_the_counter_deposit_returns_200() {
    let mut app = spawn_app().await;

    app.clear_test_db().await;
}

async fn unauthenticated_staff_over_the_counter_deposit_returns_401() {
    let mut app = spawn_app().await;

    app.clear_test_db().await;
}

async fn authenticated_staff_over_the_counter_withdrawal_returns_200() {
    let mut app = spawn_app().await;

    app.clear_test_db().await;
}

async fn unauthenticated_staff_over_the_counter_withdrawal_returns_401() {
    let mut app = spawn_app().await;

    app.clear_test_db().await;
}

async fn authenticated_customer_atm_deposit_returns_200() {
    let mut app = spawn_app().await;

    app.clear_test_db().await;
}

async fn unauthenticated_customer_atm_deposit_returns_401() {
    let mut app = spawn_app().await;

    app.clear_test_db().await;
}
