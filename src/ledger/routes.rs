use super::repo::{db_get_journal_entry, db_get_journal_entry_by_id};
use super::schemas::JournalRequest;
use crate::{
    base::error::ErrorExt,
    ledger::schemas::{JournalIdRequest, JournalResponse},
};
use actix_web::{HttpResponse, web};
use sqlx::PgPool;

#[tracing::instrument("Fetching journal entries", skip(pool))]
#[utoipa::path(get, path="/journal", responses((status=200, body=JournalResponse, description="Journal entries found"), (status=404, description="Journal entries not found")))]
pub async fn get_journal_entry(
    pool: web::Data<PgPool>,
    request: web::Query<JournalRequest>,
) -> actix_web::Result<HttpResponse> {
    let entries = db_get_journal_entry(&pool, &request.into_inner())
        .await
        .to_internal()?;

    let response: JournalResponse = entries.into();
    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument("Fetching journal entries", skip(pool))]
#[utoipa::path(get, path="/journal/{journal_id}", responses((status=200, body=JournalResponse, description="Journal entry found"), (status=404, description="Journal entry not found")))]
pub async fn get_journal_entry_by_id(
    pool: web::Data<PgPool>,
    request: web::Path<JournalIdRequest>,
) -> actix_web::Result<HttpResponse> {
    let entry = db_get_journal_entry_by_id(&pool, &request.into_inner())
        .await
        .to_internal()?;

    let response: JournalResponse = entry.into();
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_balance() {}

pub async fn get_trial_balance() {}
