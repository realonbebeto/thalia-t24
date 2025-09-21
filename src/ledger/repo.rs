use crate::base::error::DBError;
use crate::ledger::models::{IntoJournalLine, JournalEntry};
use crate::ledger::schemas::{JournalEntryLine, JournalIdRequest, JournalRequest};
use error_stack::{Report, ResultExt};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};

pub async fn db_get_journal_entry(
    pool: &PgPool,
    query: &JournalRequest,
) -> Result<Vec<JournalEntryLine>, Report<DBError>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT je.id, je.reference_id, je.description, je.created_at, jl.coa_id, jl.line_type, jl.amount_cents 
                FROM journal_entry je
                LEFT JOIN journal_line jl ON je.id=jl.journal_entry_id
                WHERE",
    );

    builder
        .push("created_date >= $1")
        .push_bind(query.start_date)
        .push("AND created_date < $1")
        .push_bind(query.end_date);

    if let Some(r_id) = &query.reference_id {
        builder.push("AND je.reference_id = $1").push_bind(r_id);
    }

    if let Some(j_ids) = &query.journal_id {
        builder.push("je.id IN (");
        let mut comma_sep = builder.separated(", ");

        for id in j_ids {
            comma_sep.push(id);
        }

        comma_sep.push_unseparated(")");
    }

    if let Some(lt) = &query.line_type {
        builder.push("AND jl.line_type = $1").push_bind(lt);
    }

    let result = builder
        .build_query_as::<JournalEntryLine>()
        .fetch_all(pool)
        .await
        .change_context(DBError::DBFault {
            message: "Error while fetching journal entries".into(),
        })?;

    Ok(result)
}

pub async fn db_get_journal_entry_by_id(
    pool: &PgPool,
    query: &JournalIdRequest,
) -> Result<Vec<JournalEntryLine>, sqlx::Error> {
    let result = sqlx::query_as::<_, JournalEntryLine>("SELECT je.id, je.reference_id, je.description, je.created_at, jl.coa_id, jl.line_type, jl.amount_cents 
                FROM journal_entry je
                LEFT JOIN journal_line jl ON je.id=jl.journal_entry_id
                WHERE je.id  = $1").bind(query.journal_id).fetch_all(pool).await?;

    Ok(result)
}

pub async fn db_add_ledger_journal_entry(
    tx: &mut Transaction<'_, Postgres>,
    journal_entry: &JournalEntry,
) -> Result<(), Report<DBError>> {
    sqlx::query("INSERT INTO journal_entry(id, user_account_id, transaction_id, transaction_ref, description) VALUES($1, $2, $3)")
        .bind(journal_entry.get_id())
        .bind(journal_entry.get_user_account_id())
        .bind(journal_entry.get_transaction_id())
        .bind(journal_entry.get_description())
        .execute(&mut **tx)
        .await.change_context(DBError::DBFault { message: "Error while inserting ledger journal entry".into() })?;

    Ok(())
}

pub async fn db_add_ledger_journal_line(
    tx: &mut Transaction<'_, Postgres>,
    journal_line: IntoJournalLine,
) -> Result<(), Report<DBError>> {
    sqlx::query(
        "INSERT INTO journal_line(id, journal_entry_id, coa_id, amount_cents, line_type) 
                        VALUES ($1, $2, $3, $4, $5), ($6, $7, $8, $9, $10)",
    )
    // Debit line
    .bind(journal_line.get_debit_line().get_id())
    .bind(journal_line.get_journal_entry_id())
    .bind(journal_line.get_debit_line().get_coa_id())
    .bind(journal_line.get_amount_cents())
    .bind(journal_line.get_debit_line().get_line_type())
    // Credit Line
    .bind(journal_line.get_credit_line().get_id())
    .bind(journal_line.get_journal_entry_id())
    .bind(journal_line.get_credit_line().get_coa_id())
    .bind(journal_line.get_amount_cents())
    .bind(journal_line.get_credit_line().get_line_type())
    .execute(&mut **tx)
    .await
    .change_context(DBError::DBFault {
        message: "Error while inserting ledger journal line entry".into(),
    })?;

    Ok(())
}
