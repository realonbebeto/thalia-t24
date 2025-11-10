use actix_web::{HttpResponse, web};

use crate::account::schemas::UserAccountCreateRequest;
use crate::account::service::AccountService;
use crate::base::StdResponse;
use crate::config::state::AppState;

#[tracing::instrument("Open customer account", skip(app_state))]
#[utoipa::path(post, path="/account", responses((status=200, body=StdResponse, description="Successfull bank account opening"), (status=409, description="Opening bank account failed")))]
pub async fn open_customer_account(
    app_state: web::Data<AppState>,
    request: web::Json<UserAccountCreateRequest>,
) -> actix_web::Result<HttpResponse> {
    let acc_service = AccountService::from(&app_state);

    acc_service
        .create_user_account(request.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Successfull bank account opening")))
}
