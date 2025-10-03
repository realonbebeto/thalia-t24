use crate::account::docs::AccountApi;
use crate::customer::docs::CustomerApi;
use crate::ledger::docs::LedgerApi;
use crate::staff::docs::StaffApi;
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
    nest((path="/staff", api=AccountApi),
        (path="/customer", api=AccountApi),
        (path="/staff", api =StaffApi), 
        (path="/customer", api=CustomerApi),
            (path="/ledger", api=LedgerApi),
            (path="/transaction", api=TransactionApi)),
    paths(crate::index::health_check)
)]
pub struct ApiDoc;
