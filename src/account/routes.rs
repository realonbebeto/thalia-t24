use crate::account::schemas::UserAccountCreateRequest;
use crate::account::service::create_user_account;
use crate::base::{StdResponse, error::ErrorExt};
use actix_web::{HttpResponse, web};
use isocountry::CountryCode;
use sqlx::PgPool;

#[tracing::instrument("Open customer account", skip(pool))]
#[utoipa::path(post, path="/account", responses((status=200, body=StdResponse, description="Successfull bank account opening"), (status=409, description="Opening bank account failed")))]
pub async fn open_customer_account(
    pool: web::Data<PgPool>,
    request: web::Json<UserAccountCreateRequest>,
) -> actix_web::Result<HttpResponse> {
    let request = request.into_inner();
    let country_code = CountryCode::for_id(request.country_code).to_badrequest()?;

    create_user_account(
        &pool,
        request.user_id,
        request.branch_id,
        request.account_id,
        request.account_class,
        country_code,
    )
    .await
    .map_err(|e| e.current_context().clone())?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Successfull bank account opening")))
}
