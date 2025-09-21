use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::transaction::routes::deposit_funds,
    crate::transaction::routes::withdraw_funds,
))]
pub struct TransactionApi;
