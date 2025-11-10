use actix_web::{HttpResponse, cookie::Cookie, http::header, web};
use uuid::Uuid;

use crate::account::{schemas::UserAccountBalance, service::AccountService};
use crate::authentication::{
    schemas::LoginRequest, service::AuthService, session_state::CustomerSession,
};
use crate::base::StdResponse;
use crate::config::state::AppState;
use crate::user::{schemas::UserRegisterRequest, service::UserService};

// Create account
#[tracing::instrument("Customer signup", skip(request, app_state))]
#[utoipa::path(post, path="/signup", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="User already exists")))]
pub async fn customer_signup(
    app_state: web::Data<AppState>,
    request: web::Json<UserRegisterRequest>,
) -> actix_web::Result<HttpResponse> {
    let user_service = UserService::from(&app_state);

    user_service.create_user(request.into_inner()).await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Profile successfully created")))
}

// activate account
#[tracing::instrument("Confirm profile")]
#[utoipa::path(get, path="/customer/confirm/{token}", responses((status=200, body=StdResponse, description="User activated successfully"), (status=409, description="User activation failed")))]
pub async fn confirm_customer(
    app_state: web::Data<AppState>,
    req: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let auth_service = AuthService::from(&app_state);

    auth_service.verify_user_email(req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Successful activation")))
}

#[tracing::instrument("Customer login", skip(app_state, session))]
#[utoipa::path(post, path="/login", responses((status=200, body=StdResponse, description="Customer login successful"), (status=401, description="Customer login unsuccessful")))]
pub async fn customer_login(
    app_state: web::Data<AppState>,
    payload: web::Json<LoginRequest>,
    session: CustomerSession,
) -> actix_web::Result<HttpResponse> {
    let auth_service = AuthService::from(&app_state);

    let (access_token, refresh_token) = auth_service
        .authenticate_user(payload.into_inner(), session)
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", access_token)))
        .cookie(
            Cookie::build("refresh_token", refresh_token)
                .http_only(true)
                .finish(),
        )
        .json(StdResponse::from("Customer Login Successful")))
}

// KYC
#[tracing::instrument("Upload user docs")]
#[utoipa::path(post, path="/{id}/kyc", responses((status=200, body=StdResponse, description="Docs uploaded successfully"), (status=409, description="Docs failed to upload")))]
pub async fn upload_user_docs() {}

#[tracing::instrument("Customer profile status")]
// Used to confirm if a user has been verified
#[utoipa::path(post, path="/user/{id}/kyc", responses((status=200, body=StdResponse, description="Successfull verification"), (status=409, description="Verification failed")))]
pub async fn customer_profile_status() {}

#[tracing::instrument("Fetch balance", skip(app_state))]
#[utoipa::path(get, path="/balance/{account_id}", responses((status=200, body=UserAccountBalance, description="Successfull balance check"), (status=409, description="Failed balance check")))]
pub async fn fetch_balances(
    app_state: web::Data<AppState>,
    account_id: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let account_service = AccountService::from(&app_state);

    let result = account_service
        .read_acc_balance(account_id.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn fetch_transactions() {}
