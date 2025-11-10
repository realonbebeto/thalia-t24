use actix_web::{HttpResponse, web};

use crate::{
    config::state::AppState,
    ledger::{
        schemas::{JournalIdRequest, JournalRequest, JournalResponse},
        service::LedgerService,
    },
};

#[tracing::instrument("Fetching journal entries", skip(app_state))]
#[utoipa::path(get, path="/journal", responses((status=200, body=JournalResponse, description="Journal entries found"), (status=404, description="Journal entries not found")))]
pub async fn journal_entry(
    app_state: web::Data<AppState>,
    request: web::Query<JournalRequest>,
) -> actix_web::Result<HttpResponse> {
    let ledger_service = LedgerService::from(&app_state);

    let response = ledger_service.journal_entry(request.into_inner()).await?;

    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument("Fetching journal entries", skip(app_state))]
#[utoipa::path(get, path="/journal/{journal_id}", responses((status=200, body=JournalResponse, description="Journal entry found"), (status=404, description="Journal entry not found")))]
pub async fn journal_entry_by_id(
    app_state: web::Data<AppState>,
    request: web::Path<JournalIdRequest>,
) -> actix_web::Result<HttpResponse> {
    let ledger_service = LedgerService::from(&app_state);
    let response = ledger_service
        .journal_entry_by_id(request.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_balance() {}

pub async fn get_trial_balance() {}
