use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::admin::routes::staff_signup,
    crate::admin::routes::staff_login,
    crate::admin::routes::create_customer_account,
    crate::admin::routes::update_customer_account,
    crate::admin::routes::create_chart_account,
    crate::admin::routes::update_chart_account,
    crate::admin::routes::create_account_type,
    crate::admin::routes::update_account_type,
))]
pub struct AdminApi;
