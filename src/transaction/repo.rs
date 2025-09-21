use crate::{
    base::error::DBError,
    transaction::models::{HeaderPairRecord, TransactionIdempotent},
    transaction::schemas::TRResponse,
};
use actix_web::{HttpResponse, http::StatusCode};
use error_stack::{Report, ResultExt};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn db_start_tx_idempotent_record(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    transaction_ref: &str,
    amount: f64,
) -> Result<u64, Report<DBError>> {
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
    .change_context(DBError::DBFault {
        message: "Error while inserting into transaction_idempotent".into(),
    })?
    .rows_affected();

    Ok(n_inserted_rows)
}

pub async fn db_save_tx_response(
    tx: &mut Transaction<'_, Postgres>,
    transaction_res: TransactionIdempotent,
) -> Result<(), Report<DBError>> {
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
    .change_context(DBError::DBFault {
        message: "Error while updating transaction idempotent response s".into(),
    })?;

    Ok(())
}

pub async fn db_get_saved_tx_response(
    pool: &PgPool,
    account_id: Uuid,
    transaction_ref: &str,
) -> Result<HttpResponse, Report<DBError>> {
    let saved_response = sqlx::query_as::<_, TRResponse>(
        "SELECT response_status_code, response_headers, response_body FROM transaction_idempotent
        WHERE account_id=$1
        AND transaction_ref=$2",
    )
    .bind(account_id)
    .bind(transaction_ref)
    .fetch_optional(pool)
    .await
    .change_context(DBError::DBFault {
        message: "Error while fetching transaction response".into(),
    })?;

    if let Some(r) = saved_response {
        let status_code =
            StatusCode::from_u16(r.response_status_code.unwrap().try_into().change_context(
                DBError::DBFault {
                    message: "Corrupted status code from DB".into(),
                },
            )?)
            .change_context(DBError::DBFault {
                message: "Error parsing integer to status code".into(),
            })?;

        let mut response = HttpResponse::build(status_code);
        for HeaderPairRecord { name, value } in r.response_headers.unwrap() {
            response.append_header((name, value));
        }

        Ok(response.body(r.response_body.unwrap()))
    } else {
        Err(Report::new(DBError::NotFound).attach(format!(
            "Saved response of account_id: {} and transaction_ref: {} not found",
            account_id, transaction_ref,
        )))
    }
}
