use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::staff::routes::staff_signup,
    crate::staff::routes::staff_login,
    crate::staff::routes::confirm_staff,
    crate::staff::routes::create_customer_account,
    crate::staff::routes::update_customer_account,
    crate::staff::routes::create_chart_account,
    crate::staff::routes::update_chart_account,
    crate::staff::routes::create_account_type,
    crate::staff::routes::update_account_type,
))]
pub struct StaffApi;
