use crate::staff::models::{ChartAccount, CoaType, CustomerAccountType};
use crate::telemetry::TraceError;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[tracing::instrument("Insert chart account to db", skip(pool))]
pub async fn db_create_chart_account(pool: &PgPool, coa: &ChartAccount) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO chart_of_account(id, code, name, coa_type, currency) VALUES($1, $2, $3, $4, $5)",
    )
    .bind(coa.get_id())
    .bind(coa.get_code())
    .bind(coa.get_name())
    .bind(coa.get_coa_type())
    .bind(coa.get_currency())
    .execute(pool)
    .await
    .trace_with("Error while inserting a chart account")?;

    Ok(())
}

#[tracing::instrument("Fetch chart account id by type from db", skip(tx))]
pub async fn db_get_coa_id_by_coa_type(
    tx: &mut Transaction<'_, Postgres>,
    coa_type: CoaType,
) -> Result<Uuid, sqlx::Error> {
    let result = sqlx::query("SELECT id FROM chart_of_account WHERE coa_type=$1")
        .bind(&coa_type)
        .fetch_optional(&mut **tx)
        .await
        .trace_with(&format!(
            "Error while fetching chart account id of: {}",
            coa_type
        ))?;

    match result {
        Some(r) => {
            let r = r.get::<Uuid, _>("id");
            Ok(r)
        }
        None => Err(sqlx::Error::RowNotFound)
            .trace_with(&format!("Chart account id of: {}  not found", coa_type)),
    }
}

#[tracing::instrument("Insert account type in db", skip(pool, acc_type))]
pub async fn db_create_account_type(
    pool: &PgPool,
    acc_type: &CustomerAccountType,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO chart_of_account(id, name, description, coa_id) VALUES($1, $2, $3, $4)",
    )
    .bind(acc_type.id)
    .bind(&acc_type.name)
    .bind(&acc_type.description)
    .bind(acc_type.coa_id)
    .execute(pool)
    .await
    .trace_with("Error while inserting account type")?;
    Ok(())
}

pub async fn db_get_coa_by_code(
    pool: &PgPool,
    code: &str,
) -> Result<Option<ChartAccount>, sqlx::Error> {
    let result = sqlx::query_as(
        "SELECT id, code, name, coa_type, currency FROM chart_of_account WHERE code=$1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .trace_with("Error while fetching coa by code")?;

    Ok(result)
}
