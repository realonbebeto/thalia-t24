use crate::transaction::models::HeaderPairRecord;
use chrono::Utc;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub enum DepositMetadata {
    Teller { id: Option<String> },
    Device { id: Option<String> },
}

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct CashDepositRequest {
    pub amount: f64,
    pub currency: String,
    pub transaction_ref: String,
    pub source: String,
    pub location_id: Uuid,
    pub notes: String,
    pub metadata: DepositMetadata,
}

#[derive(Debug, utoipa::ToSchema, serde::Serialize)]
pub struct CashResponse {
    status: String,
    transaction_id: String,
    account_id: Uuid,
    account_balance: f64,
    currency: String,
    timestamp: chrono::DateTime<Utc>,
    fees: f64,
}

impl CashResponse {
    pub fn new(
        status: &str,
        transaction_id: String,
        account_id: Uuid,
        account_balance: f64,
        currency: String,
        timestamp: chrono::DateTime<Utc>,
        fees: f64,
    ) -> Self {
        CashResponse {
            status: status.to_owned(),
            transaction_id,
            account_id,
            account_balance,
            currency,
            timestamp,
            fees,
        }
    }
}

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct WithdrawMetadata {
    pub last_four: String,
    pub auth_code: String,
}

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct CashWithdrawRequest {
    pub amount: f64,
    pub currency: String,
    pub transaction_ref: String,
    pub channel: String,
    pub location_id: String,
    pub notes: String,
    pub metadata: WithdrawMetadata,
}

#[derive(Debug)]
pub struct ErrorResponse {
    pub status: String,
    pub error_code: String,
    pub message: String,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct TRResponse {
    pub response_status_code: Option<i16>,
    pub response_headers: Option<Vec<HeaderPairRecord>>,
    pub response_body: Option<Vec<u8>>,
}
