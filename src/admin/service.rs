use crate::admin::models::{ChartAccount, CustomerAccountType};
use crate::admin::repo::{db_create_account_type, db_create_chart_account, db_get_coa_by_code};
use crate::admin::schemas::{AccountTypeRequest, ChartAccountRequest};
use crate::base::error::BaseError;
use error_stack::{Report, ResultExt};
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument("Create account type", skip(pool))]
pub async fn account_type_creation(
    pool: &PgPool,
    request: AccountTypeRequest,
) -> Result<(), Report<BaseError>> {
    let acc_type = CustomerAccountType {
        id: Uuid::now_v7(),
        coa_id: request.coa_id,
        description: request.description,
        name: request.name,
    };
    db_create_account_type(pool, &acc_type)
        .await
        .change_context(BaseError::Internal)?;

    Ok(())
}

#[tracing::instrument("Create chart account", skip(pool))]
pub async fn chart_account_creation(
    pool: &PgPool,
    request: ChartAccountRequest,
) -> Result<(), BaseError> {
    let coa = ChartAccount::new(
        Uuid::now_v7(),
        &request.code,
        request.name,
        request
            .coa_type
            .try_into()
            .change_context(BaseError::BadRequest {
                message: "Wrong chart account type".into(),
            })?,
        request.currency,
    );

    if db_get_coa_by_code(pool, &request.code)
        .await
        .change_context(BaseError::Internal)?
        .is_some()
    {
        Err(BaseError::AlreadyExists {
            message: "coa already exists".into(),
        })?
    }

    db_create_chart_account(pool, &coa)
        .await
        .change_context(BaseError::Internal)
        .attach(format!("Failed to create chart account: {:?}", coa))?;

    Ok(())
}
