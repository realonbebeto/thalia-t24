use crate::config::state::AppState;
use crate::transaction::schemas::CashDepositRequest;
use crate::transaction::service::TransactionService;
use crate::{
    base::ids::AccountId,
    transaction::schemas::{CashResponse, CashWithdrawRequest},
};
use actix_web::{HttpResponse, web};

use sqlx::PgPool;

// Payment, Withdraw, Deposit
#[tracing::instrument("Depositing funds", skip(app_state, payload))]
#[utoipa::path(get, path="/deposit", responses((status=200, body=CashResponse, description="Deposit successful"), (status=404, description="Deposit Failed")))]
pub async fn deposit_funds(
    app_state: web::Data<AppState>,
    payload: web::Json<CashDepositRequest>,
    account_id: web::ReqData<AccountId>,
) -> actix_web::Result<HttpResponse> {
    let transact_service = TransactionService::from(&app_state);

    let response = transact_service
        .fund_deposit(account_id.into_inner(), payload.into_inner())
        .await?;

    Ok(response)
}

#[tracing::instrument("Withdrawing funds")]
#[utoipa::path(get, path="/withdraw", responses((status=200, body=CashResponse, description="Deposit successful"), (status=404, description="Deposit Failed")))]
pub async fn withdraw_funds(
    pool: web::Data<PgPool>,
    request: web::Json<CashWithdrawRequest>,
) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

// Fund Transfer (Internal/External)

// Payment Processing (Bill/Utility)
