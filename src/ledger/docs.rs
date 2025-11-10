use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::ledger::routes::journal_entry_by_id,
    crate::ledger::routes::journal_entry
))]
pub struct LedgerApi;
