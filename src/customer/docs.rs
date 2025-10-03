use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::customer::routes::customer_signup,
    crate::customer::routes::confirm_customer,
    crate::customer::routes::customer_login,
    crate::customer::routes::upload_user_docs,
    crate::customer::routes::customer_profile_status,
))]
pub struct CustomerApi;
