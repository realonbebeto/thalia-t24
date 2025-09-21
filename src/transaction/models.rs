use error_stack::Report;
use getset::Getters;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum TRRefError {
    #[error("Invalid transaction reference: {message}")]
    Invalid { message: String },
}

pub struct TransactionRef(String);

impl TryFrom<String> for TransactionRef {
    type Error = Report<TRRefError>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(Report::new(TRRefError::Invalid { message: value }));
        }

        let max_length = 50;
        if value.len() > max_length {
            return Err(Report::new(TRRefError::Invalid { message: value }));
        }

        Ok(Self(value))
    }
}

impl From<TransactionRef> for String {
    fn from(value: TransactionRef) -> Self {
        value.0
    }
}

impl AsRef<str> for TransactionRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, sqlx::Type, Getters)]
#[sqlx(type_name = "header_pair")]
#[get = "pub with_prefix"]
pub struct HeaderPairRecord {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(Debug, FromRow, Getters)]
#[get = "pub with_prefix"]
pub struct TransactionIdempotent {
    account_id: Uuid,
    transaction_ref: String,
    amount_cents: u64,
    response_status_code: i16,
    response_headers: Vec<HeaderPairRecord>,
    response_body: Vec<u8>,
}

impl TransactionIdempotent {
    pub fn new(
        account_id: Uuid,
        transaction_ref: String,
        amount: f64,
        response_status_code: i16,
        response_headers: Vec<HeaderPairRecord>,
        response_body: Vec<u8>,
    ) -> Self {
        let amount_cents = (amount * 100.0) as u64;
        TransactionIdempotent {
            account_id,
            transaction_ref,
            amount_cents,
            response_status_code,
            response_headers,
            response_body,
        }
    }
}
