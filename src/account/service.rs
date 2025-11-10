use uuid::Uuid;

use crate::account::models::UserAccountEntity;
use crate::account::schemas::UserAccountCreateRequest;
use crate::base::error::{AppError, DomainError, SqlErrorExt};
use crate::config::state::AppState;
use crate::infra::pgdb::UnitofWork;
use crate::ledger::models::{CreditLine, DebitLine, IntoJournalLine, JournalEntry, LineType};
use crate::staff::models::CoaType;
use crate::transaction::service::TransactionService;

#[derive(Debug)]
pub struct AccountService<'a> {
    app_state: &'a AppState,
}

impl<'a> AccountService<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    // Create account
    #[tracing::instrument("Create user account", skip(self))]
    pub async fn create_user_account(
        &self,
        create_req: UserAccountCreateRequest,
    ) -> Result<(), AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let user_account_entity: UserAccountEntity = create_req.try_into()?;

        uow.accounts()
            .create(&user_account_entity)
            .await
            .to_app_err("Failed to create user account")?;

        let tx_service = TransactionService::from(self.app_state);

        let journal_entry = JournalEntry::new(
            user_account_entity.id,
            tx_service.generate_transaction_id(),
            "THA-001".into(),
            "THALIA account opening".into(),
        );

        let debit_coa_id = uow
            .staffs()
            .fetch_coa_id_by_coa_type(CoaType::Asset)
            .await
            .to_app_err("Failed to fetch debit_coa_id")?;

        let credit_coa_id = uow
            .staffs()
            .fetch_coa_id_by_coa_type(CoaType::Liability)
            .await
            .to_app_err("Failed to fetch credit_coa_id")?;

        let (debit_coa_id, credit_coa_id) = match (debit_coa_id, credit_coa_id) {
            (Some(dc), Some(cc)) => (dc, cc),
            (None, _) => Err(DomainError::NotFound(
                "Missing associated Debit chart account".into(),
            ))?,
            (_, None) => Err(DomainError::NotFound(
                "Missing associated Credit chart account".into(),
            ))?,
        };

        let debit_line = DebitLine::new(debit_coa_id, LineType::Debit);

        let credit_line = CreditLine::new(credit_coa_id, LineType::Credit);

        let journal_line =
            IntoJournalLine::new(*journal_entry.get_id(), 0.0, debit_line, credit_line);

        // Add O to the ledger
        uow.ledgers()
            .create_ledger_journal_entry(&journal_entry)
            .await
            .to_app_err("Failed to create journal entry")?;

        uow.ledgers()
            .create_ledger_journal_line(journal_line)
            .await
            .to_app_err("failed to create journal line")?;

        uow.accounts()
            .start_acc_balance(*journal_entry.get_user_account_id())
            .await
            .to_app_err("Failed to start account balance")?;

        uow.commit()
            .await
            .to_app_err("Failed to commit user account creation")?;

        Ok(())
    }

    pub async fn read_acc_balance(&self, user_acc_id: Uuid) -> Result<usize, AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let result = uow
            .accounts()
            .fetch_balance_by_user_account_id(user_acc_id)
            .await
            .to_app_err("Failed to read user account balance")?;

        Ok(result.map(|v| (v.amount_cents / 100) as usize).unwrap_or(0))
    }
}
