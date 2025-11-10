use actix_web::http::StatusCode;
use actix_web::{HttpResponse, body::to_bytes};
use actix_web_flash_messages::FlashMessage;
use uuid::Uuid;

use crate::base::error::{AppError, DomainError, SqlErrorExt};
use crate::base::ids::AccountId;
use crate::config::state::AppState;
use crate::infra::pgdb::UnitofWork;
use crate::ledger::models::{CreditLine, DebitLine, IntoJournalLine, JournalEntry, LineType};
use crate::staff::models::CoaType;
use crate::transaction::schemas::TRResponse;
use crate::transaction::{
    models::{HeaderPairRecord, TransactionIdempotent},
    schemas::{CashDepositRequest, CashResponse},
};

pub enum NextAction {
    StartProcessing,
    SavedTRResponse(HttpResponse),
}

pub struct TransactionService<'a> {
    app_state: &'a AppState,
}

impl<'a> TransactionService<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub fn generate_transaction_id(&self) -> String {
        let u = &Uuid::now_v7().as_u128().to_string()[..11];

        format!("THA{}", u)
    }

    pub async fn fund_deposit(
        &self,
        account_id: AccountId,
        cash_deposit: CashDepositRequest,
    ) -> Result<HttpResponse, AppError> {
        match self
            .try_transaction_process(
                &cash_deposit.transaction_ref,
                account_id.0,
                cash_deposit.amount,
            )
            .await?
        {
            NextAction::StartProcessing => {
                let response = self.deposit_entry(account_id.0, &cash_deposit).await?;
                let response = HttpResponse::Ok().json(response);
                Ok(response)
            }
            NextAction::SavedTRResponse(sr) => {
                FlashMessage::success("Deposit has already been processed").send();
                Ok(sr)
            }
        }
    }

    pub fn to_http(&self, tx_response: Option<TRResponse>) -> Result<HttpResponse, anyhow::Error> {
        match tx_response {
            Some(r) => {
                let status_code = StatusCode::from_u16(
                    r.response_status_code
                        .unwrap()
                        .try_into()
                        .map_err(|_| sqlx::Error::Protocol("Could not convert to i16".into()))?,
                )?;
                let mut response = HttpResponse::build(status_code);

                for HeaderPairRecord { name, value } in r.response_headers.unwrap() {
                    response.append_header((name, value));
                }
                Ok(response.body(r.response_body.unwrap()))
            }
            None => Err(DomainError::NotFound("Response not found".into()))?,
        }
    }

    #[tracing::instrument("Try Processing Transaction", skip(self))]
    async fn try_transaction_process(
        &self,
        transaction_ref: &str,
        account_id: Uuid,
        amount: f64,
    ) -> Result<NextAction, AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let n_inserted_rows = uow
            .transactions()
            .start_tx_idempotent_record(account_id, transaction_ref, amount)
            .await
            .to_app_err("Failed to start idempotent record")?;

        if n_inserted_rows > 0 {
            Ok(NextAction::StartProcessing)
        } else {
            let saved_response = uow
                .transactions()
                .fetch_saved_tx_response(account_id, transaction_ref)
                .await
                .to_app_err("Failed to fetch saved transaction response")?;

            let http_response = self.to_http(saved_response)?;

            Ok(NextAction::SavedTRResponse(http_response))
        }
    }

    #[tracing::instrument("Money Deposit", skip(self))]
    pub async fn deposit_entry(
        &self,
        user_account_id: Uuid,
        deposit: &CashDepositRequest,
    ) -> Result<CashResponse, AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let transaction_id = self.generate_transaction_id();

        let journal_entry = JournalEntry::new(
            user_account_id,
            transaction_id.clone(),
            deposit.transaction_ref.clone(),
            deposit.notes.clone(),
        );

        let debit_coa_id = uow
            .staffs()
            .fetch_coa_id_by_coa_type(CoaType::Asset)
            .await
            .to_app_err("Failed to fetch debit coa_id by coa type")?;

        let credit_coa_id = uow
            .staffs()
            .fetch_coa_id_by_coa_type(CoaType::Liability)
            .await
            .to_app_err("Failed to fetch credit coa_id by coa_type")?;

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

        let journal_line = IntoJournalLine::new(
            *journal_entry.get_id(),
            deposit.amount,
            debit_line,
            credit_line,
        );

        uow.ledgers()
            .create_ledger_journal_entry(&journal_entry)
            .await
            .to_app_err("Failed to create ledger entry for depost")?;

        uow.ledgers()
            .create_ledger_journal_line(journal_line)
            .await
            .to_app_err("Failed to create journal line for deposit")?;

        let cash_response = CashResponse::new(
            "success",
            transaction_id,
            *journal_entry.get_user_account_id(),
            0.0,
            "USD".into(),
            chrono::Utc::now(),
            0.0,
        );

        uow.commit()
            .await
            .to_app_err("Failed to commit funds deposit")?;

        Ok(cash_response)
    }

    #[tracing::instrument("Persist Transaction Response", skip(self))]
    pub async fn persist_transaction_response(
        &self,
        account_id: Uuid,
        amount: f64,
        transaction_ref: &str,
        response: HttpResponse,
    ) -> Result<HttpResponse, AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let (response, transaction_res) = self
            .response_to_tx_idempotent(account_id, amount, transaction_ref, response)
            .await?;

        uow.transactions()
            .save_tx_response(transaction_res)
            .await
            .to_app_err("Failed to save transaction response")?;

        let coa_id = match uow
            .staffs()
            .fetch_coa_id_by_coa_type(CoaType::Liability)
            .await
            .to_app_err("Failed to fetch coa_id")?
        {
            Some(v) => v,
            None => Err(DomainError::NotFound(
                "Missing chart of account type for liability".into(),
            ))?,
        };

        let balance = uow
            .accounts()
            .calculate_acc_balance(account_id, coa_id)
            .await
            .to_app_err("Failed to calculate the account balance")?;

        // We update balance cache for easy retrieval of account balances
        uow.accounts()
            .update_acc_balance(account_id, balance)
            .await
            .to_app_err("Failed to update account balance")?;

        uow.commit()
            .await
            .to_app_err("Failed to commit transaction response")?;

        Ok(response)
    }

    #[tracing::instrument("Convert response for saving to db", skip(self))]
    async fn response_to_tx_idempotent(
        &self,
        account_id: uuid::Uuid,
        amount: f64,
        transaction_ref: &str,
        http_res: HttpResponse,
    ) -> Result<(HttpResponse, TransactionIdempotent), anyhow::Error> {
        let (response_head, body) = http_res.into_parts();
        let body = to_bytes(body)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let status_code = response_head.status().as_u16() as i16;

        let headers = {
            let mut h = Vec::with_capacity(response_head.headers().len());
            for (name, value) in response_head.headers().iter() {
                let name = name.as_str().to_owned();
                let value = value.as_bytes().to_owned();
                h.push(HeaderPairRecord { name, value })
            }
            h
        };

        let tx_idem = TransactionIdempotent::new(
            account_id,
            transaction_ref.to_owned(),
            amount.to_owned(),
            status_code,
            headers,
            body.as_ref().to_owned(),
        );

        let http_res = response_head.set_body(body).map_into_boxed_body();

        Ok((http_res, tx_idem))
    }
}
