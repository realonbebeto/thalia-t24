use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct UserAccountCreateRequest {
    pub user_id: Uuid,
    pub branch_id: Uuid,
    pub account_id: Uuid,
    pub account_type: String,
    pub country_code: u32,
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
