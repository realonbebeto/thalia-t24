use crate::telemetry::TraceError;
use crate::{
    transaction::models::{HeaderPairRecord, TransactionIdempotent},
    transaction::schemas::TRResponse,
};
use actix_web::{HttpResponse, http::StatusCode};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument("Initialize transaction response in db")]
pub async fn db_start_tx_idempotent_record(
    tx: &mut Transaction<'_, Postgres>,
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
    .execute(&mut **tx)
    .await
    .trace_with("Error while inserting into transaction_idempotent")?
    .rows_affected();

    Ok(n_inserted_rows)
}

#[tracing::instrument("Save transaction response in db")]
pub async fn db_save_tx_response(
    tx: &mut Transaction<'_, Postgres>,
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
    .execute(&mut **tx)
    .await
    .trace_with("Error while updating transaction idempotent response")?;

    Ok(())
}

#[tracing::instrument("Fetch transaction response from db")]
pub async fn db_get_saved_tx_response(
    pool: &PgPool,
    account_id: Uuid,
    transaction_ref: &str,
) -> Result<HttpResponse, sqlx::Error> {
    let saved_response = sqlx::query_as::<_, TRResponse>(
        "SELECT response_status_code, response_headers, response_body FROM transaction_idempotent
        WHERE account_id=$1
        AND transaction_ref=$2",
    )
    .bind(account_id)
    .bind(transaction_ref)
    .fetch_optional(pool)
    .await
    .trace_with("Error while fetching transaction response")?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(
            r.response_status_code
                .unwrap()
                .try_into()
                .trace_with("Corrupted status code from DB")
                .map_err(|_| sqlx::Error::Protocol("Could not convert to i16".into()))?,
        )
        .trace_with("Error parsing integer to status code")
        .map_err(|_| sqlx::Error::Protocol("Could not parse i16".into()))?;

        let mut response = HttpResponse::build(status_code);
        for HeaderPairRecord { name, value } in r.response_headers.unwrap() {
            response.append_header((name, value));
        }

        Ok(response.body(r.response_body.unwrap()))
    } else {
        Err(sqlx::Error::RowNotFound).trace_with(&format!(
            "Saved response of account_id: {} and transaction_ref: {} not found",
            account_id, transaction_ref,
        ))?
    }
}
