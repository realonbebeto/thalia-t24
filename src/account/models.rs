use error_stack::Report;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "user_account_status", rename_all = "lowercase")]
pub enum UserAccountStatus {
    Active,
    Closed,
    Frozen,
    Pending,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
pub struct UserAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_number: String,
    pub iban: String,
    pub account_type: AccountType,
    pub coa_id: Uuid,
    pub branch_id: Uuid,
    pub currency: String,
    pub status: UserAccountStatus,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Savings,
    Checking,
    Loan,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountTypeError {
    #[error("Invalid account type: {message}")]
    Invalid { message: String },
}

impl TryFrom<String> for AccountType {
    type Error = Report<AccountTypeError>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().trim() {
            "savings" => Ok(AccountType::Savings),
            "loan" => Ok(AccountType::Loan),
            "checking" => Ok(AccountType::Checking),
            _ => Err(Report::new(AccountTypeError::Invalid { message: value })),
        }
    }
}
