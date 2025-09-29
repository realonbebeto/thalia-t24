use crate::admin::models::{ChartAccount, CoaType, CustomerAccountType};
use crate::base::error::DBError;
use error_stack::{Report, ResultExt};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[tracing::instrument("Insert chart account to db", skip(pool))]
pub async fn db_create_chart_account(
    pool: &PgPool,
    coa: &ChartAccount,
) -> Result<(), Report<DBError>> {
    sqlx::query(
        "INSERT INTO chart_account(id, code, name, coa_type, currency) VALUES($1, $2, $3, $4, $5)",
    )
    .bind(coa.id)
    .bind(&coa.code)
    .bind(&coa.name)
    .bind(&coa.coa_type)
    .bind(&coa.currency)
    .execute(pool)
    .await
    .change_context(DBError::DBFault {
        message: "Error while inserting a chart account".into(),
    })?;

    Ok(())
}

#[tracing::instrument("Fetch chart account id by type from db", skip(tx))]
pub async fn db_get_coa_id_by_coa_type(
    tx: &mut Transaction<'_, Postgres>,
    coa_type: CoaType,
) -> Result<Uuid, Report<DBError>> {
    let result = sqlx::query("SELECT coa_id FROM user_account WHERE coa_type=$1")
        .bind(&coa_type)
        .fetch_optional(&mut **tx)
        .await
        .change_context(DBError::DBFault {
            message: "Failed to fetch chart account detail".into(),
        })
        .attach(format!(
            "Error while fetching chart account id of: {}",
            coa_type
        ))?;

    match result {
        Some(r) => {
            let r = r.get::<Uuid, _>("coa_id");
            Ok(r)
        }
        None => Err(Report::new(DBError::NotFound)
            .attach(format!("Chart account id of: {}  not found", coa_type))),
    }
}

#[tracing::instrument("Insert account type in db", skip(pool, acc_type))]
pub async fn db_create_account_type(
    pool: &PgPool,
    acc_type: &CustomerAccountType,
) -> Result<(), Report<DBError>> {
    sqlx::query("INSERT INTO chart_account(id, name, description, coa_id) VALUES($1, $2, $3, $4)")
        .bind(acc_type.id)
        .bind(&acc_type.name)
        .bind(&acc_type.description)
        .bind(acc_type.coa_id)
        .execute(pool)
        .await
        .change_context(DBError::DBFault {
            message: "Error while inserting account type".into(),
        })?;
    Ok(())
}
