use crate::{
    base::error::{AppError, SqlErrorExt},
    config::state::AppState,
    infra::pgdb::UnitofWork,
    ledger::schemas::{JournalIdRequest, JournalRequest, JournalResponse},
};

pub struct LedgerService<'a> {
    app_state: &'a AppState,
}

impl<'a> LedgerService<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn journal_entry(
        &self,
        journal_req: JournalRequest,
    ) -> Result<JournalResponse, AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start Postgres uow")?;

        let entries = uow
            .ledgers()
            .fetch_journal_entry(&journal_req)
            .await
            .to_app_err("Failed to fetch journal entry")?;

        let response: JournalResponse = entries.into();

        Ok(response)
    }

    pub async fn journal_entry_by_id(
        &self,
        journal_id: JournalIdRequest,
    ) -> Result<JournalResponse, AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start Postgres uow")?;

        let entry = uow
            .ledgers()
            .fetch_journal_entry_by_id(&journal_id)
            .await
            .to_app_err("Failed to fetch journal entry by id")?;

        let response: JournalResponse = entry.into();

        Ok(response)
    }
}
