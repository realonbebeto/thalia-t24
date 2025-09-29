use crate::account::models::{AccountType, UserAccount, UserAccountStatus};
use crate::account::repo::{db_create_user_account, db_start_account_balance};
use crate::admin::{models::CoaType, repo::db_get_coa_id_by_coa_type};
use crate::base::error::{BaseError, DBError};
use crate::ledger::models::{CreditLine, DebitLine, IntoJournalLine, JournalEntry, LineType};
use crate::ledger::repo::{db_add_ledger_journal_entry, db_add_ledger_journal_line};
use crate::transaction::service::generate_transaction_id;
use error_stack::{Report, ResultExt};
use iso_currency::Country;
use isocountry::CountryCode;
use sqlx::PgPool;
use uuid::Uuid;

// Generate account number
fn generate_account_number() -> String {
    let u = &Uuid::now_v7().as_u128().to_string()[..10];

    u.to_string()
}

// Generate IBAN
fn generate_iban(country_code: &CountryCode) -> String {
    let u = &Uuid::now_v7().as_u128().to_string()[..20];

    format!("{}{}", country_code.alpha2(), u)
}

// Create account
#[tracing::instrument("Create user account", skip(pool))]
pub async fn create_user_account(
    pool: &PgPool,
    user_id: Uuid,
    branch_id: Uuid,
    coa_id: Uuid,
    account_type: AccountType,
    country_code: CountryCode,
) -> Result<(), Report<BaseError>> {
    let mut tx = pool
        .begin()
        .await
        .change_context(DBError::DBFault {
            message: "Error establishing postgres transaction".into(),
        })
        .change_context(BaseError::Internal)?;

    let user_account = UserAccount {
        id: Uuid::now_v7(),
        user_id,
        account_number: generate_account_number(),
        iban: generate_iban(&country_code),
        account_type,
        coa_id,
        branch_id,
        currency: Country::US.to_string(),
        status: UserAccountStatus::Pending,
    };
    db_create_user_account(&mut tx, &user_account)
        .await
        .change_context(BaseError::Internal)?;

    let journal_entry = JournalEntry::new(
        user_account.id,
        generate_transaction_id(),
        "THA-001".into(),
        "THALIA account opening".into(),
    );

    let debit_coa_id = db_get_coa_id_by_coa_type(&mut tx, CoaType::Asset)
        .await
        .change_context(BaseError::Internal)?;

    let debit_line = DebitLine::new(debit_coa_id, LineType::Debit);

    let credit_coa_id = db_get_coa_id_by_coa_type(&mut tx, CoaType::Liability)
        .await
        .change_context(BaseError::Internal)?;

    let credit_line = CreditLine::new(credit_coa_id, LineType::Credit);

    let journal_line = IntoJournalLine::new(*journal_entry.get_id(), 0.0, debit_line, credit_line);

    // Add O to the ledger
    db_add_ledger_journal_entry(&mut tx, &journal_entry)
        .await
        .change_context(BaseError::Internal)?;
    db_add_ledger_journal_line(&mut tx, journal_line)
        .await
        .change_context(BaseError::Internal)?;

    db_start_account_balance(&mut tx, *journal_entry.get_user_account_id())
        .await
        .change_context(BaseError::Internal)?;

    tx.commit()
        .await
        .change_context(DBError::DBFault {
            message: "Error while committing user account creation transaction".into(),
        })
        .change_context(BaseError::Internal)?;

    Ok(())
}
