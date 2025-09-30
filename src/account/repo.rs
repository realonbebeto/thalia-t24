use crate::account::models::UserAccount;
use crate::account::schemas::UserAccountBalance;
use crate::ledger::models::LineType;
use crate::telemetry::TraceError;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[tracing::instrument("Inserting accounts details on creation of user account", skip(tx))]
pub async fn db_create_user_account(
    tx: &mut Transaction<'_, Postgres>,
    user_account: &UserAccount,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO user_account(id, user_id, account_number, iban, account_type, coa_id, branch_id, currency, status) 
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)")
    .bind(user_account.id)
    .bind(user_account.user_id)
    .bind(&user_account.account_number)
    .bind(&user_account.iban)
    .bind(&user_account.account_type)
    .bind(user_account.coa_id)
    .bind(user_account.branch_id)
    .bind(&user_account.currency)
    .bind(&user_account.status)
    .execute(&mut **tx).await.trace_with("Error while inserting account details")?;

    Ok(())
}

#[tracing::instrument("Fetching chart account id by id", skip(pool))]
pub async fn db_get_coa_id_by_account_id(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let result = sqlx::query("SELECT coa_id FROM user_account WHERE id=$1")
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .trace_with("Error while fetching user account coa id")?;

    match result {
        Some(r) => {
            let r = r.get::<Uuid, _>("coa_id");
            Ok(r)
        }
        None => Err(sqlx::Error::RowNotFound)
            .trace_with(&format!("coa_id for account id: {} not found", account_id)),
    }
}

#[tracing::instrument("Fetching account balance by id", skip(pool))]
pub async fn db_get_balance_by_user_account_id(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<UserAccountBalance, sqlx::Error> {
    let result = sqlx::query("SELECT amount_cents FROM account_balance WHERE account_id=$1")
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .trace_with("Error fetching account balance")?;

    match result {
        Some(balance) => {
            let balance = balance.get::<i64, _>("amount_cents") as u64;
            let acc_balance =
                UserAccountBalance::new(account_id, balance, "USD".into(), chrono::Utc::now());
            Ok(acc_balance)
        }
        None => Err(sqlx::Error::RowNotFound).trace_with(&format!(
            "Account balance not found for account id: {}",
            account_id
        )),
    }
}

#[tracing::instrument("Initialize account balance cache", skip(tx))]
pub async fn db_start_account_balance(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO account_balance(account_id, amount_cents) VALUES($1, $2)")
        .bind(account_id)
        .bind(0)
        .execute(&mut **tx)
        .await
        .trace_with("Error while sarting account balance cache record")?;

    Ok(())
}

#[tracing::instrument("Update account balance cache", skip(tx))]
pub async fn db_update_account_balance(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    amount: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE account_balance
    SET amount_cents WHERE account_id=$1",
    )
    .bind(amount)
    .bind(account_id)
    .execute(&mut **tx)
    .await
    .trace_with("Error while updating account balance")?;

    Ok(())
}

#[tracing::instrument("Calculate account balance", skip(tx))]
pub async fn db_calculate_account_balance(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    coa_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "SELECT SUM(COALESCE(CASE jl.line_type 
                        WHEN $1 THEN jl.amount_cents
                        WHEN $2  THEN -jl.amount_cents
                        END, 0)) AS balance_cents
                    FROM journal_entry je JOIN journal_line jl ON je.id = jl.journal_entry_id 
                    WHERE je.user_account_id=$3
                    AND jl.coa_id = $3;",
    )
    .bind(LineType::Credit)
    .bind(LineType::Debit)
    .bind(account_id)
    .bind(coa_id)
    .fetch_one(&mut **tx)
    .await
    .trace_with("Error while calculating user account balance")?;

    let result: i64 = result.get("balance_cents");

    Ok(result)
}
