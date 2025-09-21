use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::customer::routes::customer_signup,
    crate::customer::routes::activate_profile,
    crate::customer::routes::upload_user_docs,
    crate::customer::routes::customer_profile_status,
    crate::customer::routes::open_customer_account
))]
pub struct CustomerApi;
