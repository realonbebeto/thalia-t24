use actix_web::{HttpResponse, cookie::Cookie, http::header, web};

use crate::authentication::{StaffSession, schemas::LoginRequest, service::AuthService};
use crate::base::StdResponse;
use crate::config::state::AppState;
use crate::staff::{
    schemas::{AccountTypeRequest, ChartAccountRequest},
    service::StaffService,
};
use crate::user::{schemas::UserRegisterRequest, service::UserService};

#[tracing::instrument("Staff signup", skip(app_state, request), fields(username=%request.username, user_email=%request.email))]
#[utoipa::path(post, path="/signup", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="User already exists")))]
// Signup staff accounts
pub async fn staff_signup(
    app_state: web::Data<AppState>,
    request: web::Json<UserRegisterRequest>,
) -> actix_web::Result<HttpResponse> {
    let user_service = UserService::from(&app_state);
    user_service.create_user(request.into_inner()).await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Customer created successfully")))
}

// activate account
#[tracing::instrument("Confirm profile", skip(app_state, req))]
#[utoipa::path(get, path="/staff/confirm/{token}", responses((status=200, body=StdResponse, description="User activated successfully"), (status=409, description="User activation failed")))]
pub async fn confirm_staff(
    app_state: web::Data<AppState>,
    req: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let auth_service = AuthService::from(&app_state);

    auth_service.verify_user_email(req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Successful confirmation")))
}

#[tracing::instrument("Staff login", skip(app_state, payload, session))]
#[utoipa::path(post, path="/login", responses((status=200, body=StdResponse, description="Staff login successful"), (status=401, description="Staff login unsuccessful")))]

pub async fn staff_login(
    app_state: web::Data<AppState>,
    payload: web::Json<LoginRequest>,
    session: StaffSession,
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
        .json(StdResponse::from("Staff Login Successful")))
}

// A Staff can create an account for a customer
#[tracing::instrument("Staff opening customer account", skip(app_state, payload), fields(username=%payload.username, user_email=%payload.email))]
#[utoipa::path(post, path="/user/signup", responses((status=200, body=StdResponse, description="Customer account created successfully"), (status=409, description="User already exists")))]
pub async fn create_customer_account(
    app_state: web::Data<AppState>,
    payload: web::Json<UserRegisterRequest>,
) -> actix_web::Result<HttpResponse> {
    let user_service = UserService::from(&app_state);

    user_service.create_user(payload.into_inner()).await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Customer account created successfully")))
}

#[tracing::instrument("Staff updating customer profile")]
#[utoipa::path(put, path="/user/", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="User already exists")))]
pub async fn update_customer_account() {}

#[tracing::instrument("Staff creating new chart account", skip(app_state, payload))]
#[utoipa::path(post, path="/coa", responses((status=200, body=StdResponse, description="chart account created successfully"), (status=409, description="Chart account creation failed")))]
pub async fn create_chart_account(
    app_state: web::Data<AppState>,
    payload: web::Json<ChartAccountRequest>,
) -> actix_web::Result<HttpResponse> {
    let staff_service = StaffService::from(&app_state);

    staff_service
        .chart_account_creation(payload.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Chart account created successfully")))
}

#[tracing::instrument("Staff creating new chart account")]
#[utoipa::path(put, path="/coa", responses((status=200, body=StdResponse, description="chart account created successfully"), (status=409, description="Chart account creation failed")))]
pub async fn update_chart_account() {}

#[tracing::instrument("Staff creating account type", skip(app_state))]
#[utoipa::path(post, path="/account/type", responses((status=200, body=StdResponse, description="Account type created successfully"), (status=409, description="Account type creation failed")))]
pub async fn create_account_type(
    app_state: web::Data<AppState>,
    payload: web::Json<AccountTypeRequest>,
) -> actix_web::Result<HttpResponse> {
    let staff_service = StaffService::from(&app_state);

    staff_service
        .account_type_creation(payload.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Account type created successfully")))
}

#[tracing::instrument("Staff creating account type")]
#[utoipa::path(put, path="/account/type", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="Account type update failed")))]
pub async fn update_account_type() {}
