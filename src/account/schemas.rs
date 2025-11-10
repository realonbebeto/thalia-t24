use chrono::Utc;
use iso_currency::Currency;
use isocountry::CountryCode;
use uuid::Uuid;

use crate::{
    account::models::{UserAccountEntity, UserAccountStatus},
    base::error::ValidationError,
};

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct UserAccountCreateRequest {
    pub user_id: Uuid,
    pub branch_id: Uuid,
    pub coa_id: Uuid,
    pub account_class: Uuid,
    pub country_code: u32,
}

impl TryFrom<UserAccountCreateRequest> for UserAccountEntity {
    type Error = anyhow::Error;

    fn try_from(value: UserAccountCreateRequest) -> Result<Self, Self::Error> {
        // Generate account number
        fn generate_account_number() -> String {
            let u = &Uuid::now_v7().as_u128().to_string()[..10];

            u.to_string()
        }

        // Generate IBAN
        fn generate_iban(country_code: &CountryCode) -> String {
            let u = &Uuid::now_v7().as_u128().to_string()[..20];

            format!("{}{}", country_code.alpha2(), u)
        }

        let (country_code, currency) = {
            let code = CountryCode::for_id(value.country_code);
            let currency = Currency::from_numeric(value.country_code as u16);

            match (code, currency) {
                (Ok(code), Some(curr)) => (code, curr),
                _ => Err(anyhow::anyhow!(ValidationError::InvalidValue {
                    field: "country_code".into(),
                    reason: "Unknown country ISO 3166 code".into()
                }))?,
            }
        };

        let account_number = generate_account_number();

        let iban = generate_iban(&country_code);

        Ok(Self {
            id: Uuid::now_v7(),
            user_id: value.user_id,
            account_number,
            iban,
            account_class: value.account_class,
            coa_id: value.coa_id,
            branch_id: value.branch_id,
            currency: currency.code().to_string(),
            status: UserAccountStatus::Pending,
        })
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserAccountBalance {
    account_id: Uuid,
    balance: u64,
    currency: String,
    timestamp: chrono::DateTime<Utc>,
}

impl UserAccountBalance {
    pub fn new(
        account_id: Uuid,
        balance: u64,
        currency: String,
        timestamp: chrono::DateTime<Utc>,
    ) -> Self {
        UserAccountBalance {
            account_id,
            balance,
            currency,
            timestamp,
        }
    }
}
