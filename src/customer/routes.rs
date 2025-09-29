use crate::account::repo::db_get_balance_by_user_account_id;
use crate::account::service::create_user_account;
use crate::account::{
    models::AccountType,
    schemas::{UserAccountBalance, UserAccountCreateRequest},
};
use crate::authentication::schemas::ActivateExpiryTime;
use crate::authentication::{
    create_activate_token, create_token,
    schemas::{Credentials, DefaultPassword, LoginRequest, SecretKey},
    session_state::{CustomerSession, SessionState},
    validate_activate_token, validate_credentials,
};
use crate::base::StdResponse;
use crate::base::error::{BaseError, ErrorExt};
use crate::config::Expiration;
use crate::user::repo::{db_activate_user, db_create_user};
use crate::user::{models::User, schemas::UserCreateRequest};
use actix_web::{HttpResponse, cookie::Cookie, http::header, web};
use actix_web_flash_messages::FlashMessage;
use chrono::offset::Utc;
use isocountry::CountryCode;
use sqlx::PgPool;
use uuid::Uuid;

// Create account
#[tracing::instrument("Customer signup")]
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
        return Ok(HttpResponse::BadRequest().json(StdResponse::from("You need to be 18 or older to use this service. Please try again when you meet the age requirement.")));
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

    Ok(HttpResponse::Ok().json(StdResponse::from("Profile successfully created")))
}

// activate account
#[tracing::instrument("Activate profile")]
#[utoipa::path(get, path="/activate/{token}", responses((status=200, body=StdResponse, description="User activated successfully"), (status=409, description="User activation failed")))]
pub async fn activate_profile(
    pool: web::Data<PgPool>,
    req: web::Path<String>,
    secret_key: web::Data<SecretKey>,
) -> actix_web::Result<HttpResponse> {
    let (user_email, _) =
        validate_activate_token(&req.into_inner(), &secret_key.into_inner().0).to_internal()?;

    db_activate_user(&pool, &user_email).await.to_internal()?;

    Ok(HttpResponse::Ok().json(StdResponse::from("Successful activation")))
}

#[tracing::instrument("Customer login", skip(payload, pool, session))]
#[utoipa::path(post, path="/login", responses((status=200, body=StdResponse, description="Customer login successful"), (status=401, description="Customer login unsuccessful")))]
pub async fn customer_login(
    pool: web::Data<PgPool>,
    secret_key: web::Data<SecretKey>,
    default_pass: web::Data<DefaultPassword>,
    expiration: web::Data<Expiration>,
    payload: web::Json<LoginRequest>,
    session: CustomerSession,
) -> actix_web::Result<HttpResponse> {
    let secret_key = &secret_key.into_inner().0;

    if payload.non_empty_email_username() {
        return Ok(HttpResponse::BadRequest().json(StdResponse::from("Email/Username is empty")));
    }

    let credentials =
        Credentials::from(payload.into_inner(), &default_pass.into_inner().0).to_badrequest()?;

    match validate_credentials(&pool, credentials).await {
        Ok(customer_id) => {
            tracing::Span::current().record("customer_id", tracing::field::display(customer_id));

            session.renew();

            session
                .insert_sesh_id(customer_id)
                .map_err(|_| BaseError::internal())?;

            FlashMessage::success("Customer Authorized").send();

            let access_token =
                create_token(customer_id, expiration.access_token_expire_secs, secret_key)
                    .to_internal()?;

            let refresh_token = create_token(
                customer_id,
                expiration.refresh_token_expire_secs,
                secret_key,
            )
            .to_internal()?;

            Ok(HttpResponse::Ok()
                .insert_header((header::AUTHORIZATION, format!("Bearer {}", access_token)))
                .cookie(
                    Cookie::build("refresh_token", refresh_token)
                        .http_only(true)
                        .finish(),
                )
                .json(StdResponse::from("Customer Login Successful")))
        }

        Err(e) => match e.current_context() {
            BaseError::InvalidCredentials { message } => {
                FlashMessage::info(format!("Customer login unsuccessful: {}", message)).send();
                Ok(HttpResponse::Unauthorized().json(StdResponse::from(message)))
            }
            _ => {
                FlashMessage::error("Internal Server Error").send();
                Ok(HttpResponse::InternalServerError()
                    .json(StdResponse::from("Internal Server Error")))
            }
        },
    }
}

// KYC
#[tracing::instrument("Upload user docs")]
#[utoipa::path(post, path="/{id}/kyc", responses((status=200, body=StdResponse, description="Docs uploaded successfully"), (status=409, description="Docs failed to upload")))]
pub async fn upload_user_docs() {}

#[tracing::instrument("Customer profile status")]
// Used to confirm if a user has been verified
#[utoipa::path(post, path="/user/{id}/kyc", responses((status=200, body=StdResponse, description="Successfull verification"), (status=409, description="Verification failed")))]
pub async fn customer_profile_status() {}

#[tracing::instrument("Open customer account", skip(pool))]
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

    Ok(HttpResponse::Ok().json(StdResponse::from("Successfull bank account opening")))
}

#[tracing::instrument("Fetch balance", skip(pool))]
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
