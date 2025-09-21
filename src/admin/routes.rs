use crate::base::error::ErrorExt;
use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::admin::schemas::{AccountTypeRequest, ChartAccountRequest};
use crate::admin::service::{account_type_creation, chart_account_creation};
use crate::authentication::{generate_activate_token, repo::db_store_token};
use crate::base::StdResponse;
use crate::user::repo::{db_create_user, db_get_user};
use crate::user::{models::User, schemas::UserCreateRequest};

#[tracing::instrument("Staff signup", skip(pool, request), fields(username=%request.username, user_email=%request.email))]
#[utoipa::path(post, path="/signup", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="User already exists")))]
// Signup admin accounts
pub async fn staff_signup(
    pool: web::Data<PgPool>,
    request: web::Json<UserCreateRequest>,
) -> actix_web::Result<HttpResponse> {
    let user: User = request.into_inner().try_into().to_badrequest()?;

    if db_get_user(&pool, user.email.as_ref())
        .await
        .to_internal()?
        .is_some()
    {
        return Ok(HttpResponse::Conflict().json(StdResponse {
            message: "User already exists",
        }));
    }

    let mut tx = pool.begin().await.to_internal()?;

    db_create_user(&mut tx, &user).await.to_internal()?;

    // Generate activate token
    let activate_token = generate_activate_token();

    db_store_token(&mut tx, &activate_token, user.id, user.email.as_ref())
        .await
        .to_internal()?;

    tx.commit().await.to_internal()?;

    // TODO Send Email

    Ok(HttpResponse::Ok().json(StdResponse {
        message: "User created successfully",
    }))
}

// An admin can create an account for a customer
#[tracing::instrument("Staff opening customer account", skip(pool, request), fields(username=%request.username, user_email=%request.email))]
#[utoipa::path(post, path="/user/signup", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="User already exists")))]
pub async fn create_customer_account(
    pool: web::Data<PgPool>,
    request: web::Json<UserCreateRequest>,
) -> actix_web::Result<HttpResponse> {
    let user: User = request.into_inner().try_into().to_badrequest()?;

    if db_get_user(&pool, user.email.as_ref())
        .await
        .to_internal()?
        .is_some()
    {
        return Ok(HttpResponse::Conflict().json(StdResponse {
            message: "User already exists",
        }));
    }

    let mut tx = pool.begin().await.to_internal()?;

    db_create_user(&mut tx, &user).await.to_internal()?;

    // Generate activate token
    let activate_token = generate_activate_token();

    db_store_token(&mut tx, &activate_token, user.id, user.email.as_ref())
        .await
        .to_internal()?;

    // Create user_account

    tx.commit().await.to_internal()?;

    // TODO Send Email
    Ok(HttpResponse::Ok().json(StdResponse {
        message: "User created successfully",
    }))
}

#[tracing::instrument("Staff updating customer profile")]
#[utoipa::path(put, path="/user/", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="User already exists")))]
pub async fn update_customer_account() {}

#[tracing::instrument("Staff creating new chart account", skip(pool, request))]
#[utoipa::path(post, path="/coa", responses((status=200, body=StdResponse, description="chart account created successfully"), (status=409, description="Chart account creation failed")))]
pub async fn create_chart_account(
    pool: web::Data<PgPool>,
    request: web::Json<ChartAccountRequest>,
) -> actix_web::Result<HttpResponse> {
    chart_account_creation(&pool, request.into_inner())
        .await
        .to_badrequest()?;
    Ok(HttpResponse::Ok().json(StdResponse {
        message: "Chart account created successfully",
    }))
}

#[tracing::instrument("Staff creating new chart account")]
#[utoipa::path(put, path="/coa", responses((status=200, body=StdResponse, description="chart account created successfully"), (status=409, description="Chart account creation failed")))]
pub async fn update_chart_account() {}

#[tracing::instrument("Staff creating account type")]
#[utoipa::path(post, path="/account/type", responses((status=200, body=StdResponse, description="Account type created successfully"), (status=409, description="Account type creation failed")))]
pub async fn create_account_type(
    pool: web::Data<PgPool>,
    request: web::Json<AccountTypeRequest>,
) -> actix_web::Result<HttpResponse> {
    account_type_creation(&pool, request.into_inner())
        .await
        .to_internal()?;
    Ok(HttpResponse::Ok().json(StdResponse {
        message: "Account type created successfully",
    }))
}

#[tracing::instrument("Staff creating account type")]
#[utoipa::path(put, path="/account/type", responses((status=200, body=StdResponse, description="User created successfully"), (status=409, description="Account type update failed")))]
pub async fn update_account_type() {}
