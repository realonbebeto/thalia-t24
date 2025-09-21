use crate::account::repo::db_get_balance_by_user_account_id;
use crate::account::service::create_user_account;
use crate::account::{
    models::AccountType,
    schemas::{UserAccountBalance, UserAccountCreateRequest},
};
use crate::authentication::schemas::ActivateExpiryTime;
use crate::authentication::{create_activate_token, schemas::SecretKey, validate_activate_token};
use crate::base::StdResponse;
use crate::base::error::ErrorExt;
use crate::user::repo::{db_activate_user, db_create_user};
use crate::user::{models::User, schemas::UserCreateRequest};
use actix_web::{HttpResponse, web};
use chrono::offset::Utc;
use isocountry::CountryCode;
use sqlx::PgPool;
use uuid::Uuid;

// Create account
#[utoipa::path(post, path="/signup", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="User already exists")))]
pub async fn customer_signup(
    pool: web::Data<PgPool>,
    request: web::Json<UserCreateRequest>,
    secret_key: web::Data<SecretKey>,
    expiry: web::Data<ActivateExpiryTime>,
) -> actix_web::Result<HttpResponse> {
    let user: User = request.into_inner().try_into().to_badrequest()?;
    // Check if a user is age > 18
    let today = Utc::now().date_naive();
    if (today - user.date_of_birth).num_days() < 18 * 365 {
        return Ok(HttpResponse::BadRequest().json(StdResponse{message: "You need to be 18 or older to use this service. Please try again when you meet the age requirement."}));
    }

    let mut tx = pool.begin().await.to_internal()?;
    db_create_user(&mut tx, &user).await.to_internal()?;

    tx.commit().await.to_internal()?;

    // Create activate token
    let _token = create_activate_token(
        user.id,
        user.email.as_ref(),
        expiry.into_inner().0,
        &secret_key.into_inner().0,
    )
    .to_internal()?;

    // TODO Send Activate Link

    Ok(HttpResponse::Ok().json(StdResponse {
        message: "Profile successfully created",
    }))
}

// activate account
#[utoipa::path(get, path="/activate/{token}", responses((status=200, body=StdResponse, description="User activated successfully"), (status=409, description="User activation failed")))]
pub async fn activate_profile(
    pool: web::Data<PgPool>,
    req: web::Path<String>,
    secret_key: web::Data<SecretKey>,
) -> actix_web::Result<HttpResponse> {
    let (user_email, _) =
        validate_activate_token(&req.into_inner(), &secret_key.into_inner().0).to_internal()?;

    db_activate_user(&pool, &user_email).await.to_internal()?;

    Ok(HttpResponse::Ok().json(StdResponse {
        message: "Successful activation",
    }))
}

// KYC
#[utoipa::path(post, path="/{id}/kyc", responses((status=200, body=StdResponse, description="Docs uploaded successfully"), (status=409, description="Docs failed to upload")))]
pub async fn upload_user_docs() {}

// Used to confirm if a user has been verified
#[utoipa::path(post, path="/user/{id}/kyc", responses((status=200, body=StdResponse, description="Successfull verification"), (status=409, description="Verification failed")))]
pub async fn customer_profile_status() {}

#[utoipa::path(post, path="/account", responses((status=200, body=StdResponse, description="Successfull bank account opening"), (status=409, description="Opening bank account failed")))]
pub async fn open_customer_account(
    pool: web::Data<PgPool>,
    request: web::Json<UserAccountCreateRequest>,
) -> actix_web::Result<HttpResponse> {
    let request = request.into_inner();
    let country_code = CountryCode::for_id(request.country_code).to_badrequest()?;
    let account_type: AccountType = request.account_type.try_into().to_badrequest()?;

    create_user_account(
        &pool,
        request.user_id,
        request.branch_id,
        request.account_id,
        account_type,
        country_code,
    )
    .await
    .to_internal()?;

    Ok(HttpResponse::Ok().json(StdResponse {
        message: "Successfull bank account opening",
    }))
}

#[utoipa::path(get, path="/balance/{account_id}", responses((status=200, body=UserAccountBalance, description="Successfull balance check"), (status=409, description="Failed balance check")))]
pub async fn fetch_balances(
    pool: web::Data<PgPool>,
    account_id: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let account_id = account_id.into_inner();
    let response = db_get_balance_by_user_account_id(&pool, account_id)
        .await
        .to_internal()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn fetch_transactions() {}
