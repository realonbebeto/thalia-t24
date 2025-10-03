use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(crate::account::routes::open_customer_account))]
pub struct AccountApi;
