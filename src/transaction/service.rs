use crate::account::repo::{db_calculate_account_balance, db_update_account_balance};
use crate::admin::{models::CoaType, repo::db_get_coa_id_by_coa_type};
use crate::base::error::{BaseError, ErrorExt};
use crate::telemetry::TraceError;
use crate::transaction::repo::db_save_tx_response;
use crate::transaction::repo::{db_get_saved_tx_response, db_start_tx_idempotent_record};
use crate::transaction::schemas::{CashDepositRequest, CashResponse};
use crate::transaction::util::response_to_tx_idempotent;
use crate::{
    ledger::models::{CreditLine, DebitLine, IntoJournalLine, JournalEntry, LineType},
    ledger::repo::{db_add_ledger_journal_entry, db_add_ledger_journal_line},
};
use actix_web::HttpResponse;
use error_stack::{Report, ResultExt};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub fn generate_transaction_id() -> String {
    let u = &Uuid::now_v7().as_u128().to_string()[..11];

    format!("THA{}", u)
}

pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    SavedTRResponse(HttpResponse),
}

#[tracing::instrument("Try Processing Transaction")]
pub async fn try_transaction_process(
    pool: &PgPool,
    transaction_ref: &str,
    account_id: Uuid,
    amount: f64,
) -> Result<NextAction, Report<BaseError>> {
    let mut tx = pool
        .begin()
        .await
        .trace_with("Error establishing postgres transaction")
        .change_context(BaseError::Internal)?;

    let n_inserted_rows =
        db_start_tx_idempotent_record(&mut tx, account_id, transaction_ref, amount)
            .await
            .change_context(BaseError::Internal)?;

    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing(tx))
    } else {
        let saved_response = db_get_saved_tx_response(pool, account_id, transaction_ref)
            .await
            .change_context(BaseError::Internal)?;

        Ok(NextAction::SavedTRResponse(saved_response))
    }
}

#[tracing::instrument("Money Deposit")]
pub async fn deposit_entry(
    tx: &mut Transaction<'_, Postgres>,
    user_account_id: Uuid,
    deposit: &CashDepositRequest,
) -> Result<CashResponse, Report<BaseError>> {
    let transaction_id = generate_transaction_id();

    let journal_entry = JournalEntry::new(
        user_account_id,
        transaction_id.clone(),
        deposit.transaction_ref.clone(),
        deposit.notes.clone(),
    );

    let debit_coa_id = db_get_coa_id_by_coa_type(tx, CoaType::Asset)
        .await
        .change_context(BaseError::Internal)?;

    let debit_line = DebitLine::new(debit_coa_id, LineType::Debit);

    let credit_coa_id = db_get_coa_id_by_coa_type(tx, CoaType::Liability)
        .await
        .change_context(BaseError::Internal)?;

    let credit_line = CreditLine::new(credit_coa_id, LineType::Credit);

    let journal_line = IntoJournalLine::new(
        *journal_entry.get_id(),
        deposit.amount,
        debit_line,
        credit_line,
    );

    db_add_ledger_journal_entry(tx, &journal_entry)
        .await
        .change_context(BaseError::Internal)?;
    db_add_ledger_journal_line(tx, journal_line)
        .await
        .change_context(BaseError::Internal)?;

    let cash_response = CashResponse::new(
        "success",
        transaction_id,
        *journal_entry.get_user_account_id(),
        0.0,
        "USD".into(),
        chrono::Utc::now(),
        0.0,
    );

    Ok(cash_response)
}

#[tracing::instrument("Persist Transaction Response")]
pub async fn persist_transaction_response(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    amount: f64,
    transaction_ref: &str,
    response: HttpResponse,
) -> Result<HttpResponse, Report<BaseError>> {
    let (response, transaction_res) =
        response_to_tx_idempotent(account_id, amount, transaction_ref, response)
            .await
            .to_internal()?;

    db_save_tx_response(transaction, transaction_res)
        .await
        .to_internal()?;

    let coa_id = db_get_coa_id_by_coa_type(transaction, CoaType::Liability)
        .await
        .change_context(BaseError::Internal)?;

    let balance = db_calculate_account_balance(transaction, account_id, coa_id)
        .await
        .change_context(BaseError::Internal)?;

    // We update balance cache for easy retrieval of account balances

    db_update_account_balance(transaction, account_id, balance)
        .await
        .change_context(BaseError::Internal)?;

    Ok(response)
}
