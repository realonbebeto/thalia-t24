use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::ledger::routes::get_journal_entry_by_id,
    crate::ledger::routes::get_journal_entry
))]
pub struct LedgerApi;
