use crate::admin::models::{ChartAccount, CustomerAccountType};
use crate::admin::repo::{db_create_account_type, db_create_chart_account};
use crate::admin::schemas::{AccountTypeRequest, ChartAccountRequest};
use crate::base::error::BaseError;
use error_stack::{Report, ResultExt};
use sqlx::PgPool;
use uuid::Uuid;

pub fn generate_code() -> String {
    let u = &Uuid::now_v7().as_u128().to_string()[..5];

    u.to_string()
}

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

pub async fn chart_account_creation(
    pool: &PgPool,
    request: ChartAccountRequest,
) -> Result<(), Report<BaseError>> {
    let code = generate_code();

    let coa = ChartAccount {
        id: Uuid::now_v7(),
        name: request.name,
        coa_type: request
            .coa_type
            .try_into()
            .change_context(BaseError::BadRequest {
                message: "Wrong chart account type".into(),
            })?,
        code,
        currency: request.currency,
    };

    db_create_chart_account(pool, &coa)
        .await
        .change_context(BaseError::Internal)
        .attach(format!("Failed to create chart account: {:?}", coa))?;

    Ok(())
}
