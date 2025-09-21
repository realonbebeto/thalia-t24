use crate::base::error::ErrorExt;
use crate::transaction::service::{NextAction, deposit_entry, try_transaction_process};
use crate::{
    base::ids::AccountId,
    transaction::schemas::{CashDepositRequest, CashResponse, CashWithdrawRequest},
};
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;

// Payment, Withdraw, Deposit
#[tracing::instrument("Depositing funds", skip(pool, request))]
#[utoipa::path(get, path="/deposit", responses((status=200, body=CashResponse, description="Deposit successful"), (status=404, description="Deposit Failed")))]
pub async fn deposit_funds(
    pool: web::Data<PgPool>,
    request: web::Json<CashDepositRequest>,
    account_id: web::ReqData<AccountId>,
) -> actix_web::Result<HttpResponse> {
    let account_id = account_id.0;
    let request = request.into_inner();

    // let mut tx = pool.begin().await.to_internal()?;

    let mut transaction =
        match try_transaction_process(&pool, &request.transaction_ref, account_id, request.amount)
            .await
            .to_internal()?
        {
            NextAction::StartProcessing(tx) => tx,
            NextAction::SavedTRResponse(sr) => {
                FlashMessage::success("Deposit has already been processed").send();
                return Ok(sr);
            }
        };

    let cash_response = deposit_entry(&mut transaction, account_id, &request)
        .await
        .to_internal()?;

    let response = HttpResponse::Ok().json(cash_response);

    // Persist

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
