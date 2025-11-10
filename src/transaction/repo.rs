use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{transaction::models::TransactionIdempotent, transaction::schemas::TRResponse};

pub struct TransactionRepository<'a, 'b> {
    pool: &'a PgPool,
    tx: &'b mut Transaction<'a, Postgres>,
}

impl<'a, 'b> TransactionRepository<'a, 'b> {
    pub fn from(pool: &'a PgPool, tx: &'b mut Transaction<'a, Postgres>) -> Self {
        Self { pool, tx }
    }

    #[tracing::instrument("Initialize transaction response in db", skip(self))]
    pub async fn start_tx_idempotent_record(
        &mut self,
        account_id: Uuid,
        transaction_ref: &str,
        amount: f64,
    ) -> Result<u64, sqlx::Error> {
        let amount_cents = (amount * 100.0) as i64;
        let n_inserted_rows = sqlx::query(
            "INSERT INTO transaction_idempotent (account_id, transaction_ref, amount_cents)
                    VALUES ($1, $2, $3)",
        )
        .bind(account_id)
        .bind(transaction_ref)
        .bind(amount_cents)
        .execute(&mut **self.tx)
        .await?
        .rows_affected();

        Ok(n_inserted_rows)
    }

    #[tracing::instrument("Save transaction response in db", skip(self))]
    pub async fn save_tx_response(
        &mut self,
        transaction_res: TransactionIdempotent,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE transaction_idempotent
                    SET response_status_code=$1,
                        response_headers=$2,
                        response_body=$3
                    WHERE account_id=$4
                    AND transaction_ref=$5",
        )
        .bind(transaction_res.get_response_status_code())
        .bind(transaction_res.get_response_headers())
        .bind(transaction_res.get_response_body())
        .bind(transaction_res.get_account_id())
        .bind(transaction_res.get_transaction_ref())
        .execute(&mut **self.tx)
        .await?;

        Ok(())
    }

    #[tracing::instrument("Fetch transaction response from db", skip(self))]
    pub async fn fetch_saved_tx_response(
        &self,
        account_id: Uuid,
        transaction_ref: &str,
    ) -> Result<Option<TRResponse>, sqlx::Error> {
        let saved_response = sqlx::query_as::<_, TRResponse>(
        "SELECT response_status_code, response_headers, response_body FROM transaction_idempotent
        WHERE account_id=$1
        AND transaction_ref=$2",
        )
        .bind(account_id)
        .bind(transaction_ref)
        .fetch_optional(self.pool)
        .await?;

        Ok(saved_response)
    }
}
