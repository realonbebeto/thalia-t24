use crate::admin::docs::AdminApi;
use crate::customer::docs::CustomerApi;
use crate::ledger::docs::LedgerApi;
use crate::transaction::docs::TransactionApi;
use utoipa::OpenApi;
// API Configuration and Documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Thalia-T24 API",
        description = "Thalia-T24 is a fictional greenfield rewrite of Temenos T24",
        version = "0.1.0"
    ),
    nest((path="/admin", api =AdminApi), (path="/ledger", api=LedgerApi), (path="/customer", api=CustomerApi), (path="/transaction", api=TransactionApi)),
    paths(crate::index::health_check)
)]
pub struct ApiDoc;
