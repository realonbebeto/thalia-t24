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
    pub account_class: Uuid,
    pub coa_id: Uuid,
    pub branch_id: Uuid,
    pub currency: String,
    pub status: UserAccountStatus,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "account_kind", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Deposit,
    Investment,
    Loan,
    Specialty,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountKindError {
    #[error("Invalid account type: {message}")]
    Invalid { message: String },
}

impl TryFrom<String> for AccountKind {
    type Error = Report<AccountKindError>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().trim() {
            "deposit" => Ok(AccountKind::Deposit),
            "investment" => Ok(AccountKind::Investment),
            "loan" => Ok(AccountKind::Loan),
            "specialty" => Ok(AccountKind::Specialty),
            _ => Err(Report::new(AccountKindError::Invalid { message: value })),
        }
    }
}

#[derive(Debug, sqlx::FromRow, getset::CloneGetters)]
#[get_clone = "pub with_prefix"]
pub struct BehaviorPolicy {
    default_interest_rate: u32,
    default_min_balance: u32,
}

impl BehaviorPolicy {
    pub fn new(default_interest_rate: u32, default_min_balance: u32) -> Self {
        Self {
            default_interest_rate,
            default_min_balance,
        }
    }
}

#[derive(Debug, sqlx::FromRow, getset::Getters)]
#[get = "pub with_prefix"]
pub struct AccountClass {
    id: Uuid,
    kind: AccountKind,
    code: String,
    name: String,
    description: String,
    coa_id: Uuid,
    default_interest_rate: u32,
    default_min_balance: u32,
}

impl AccountClass {
    pub fn new(
        id: Uuid,
        kind: AccountKind,
        code: &str,
        name: &str,
        description: &str,
        coa_id: Uuid,
        behave_policy: &BehaviorPolicy,
    ) -> Self {
        Self {
            id,
            kind,
            code: code.into(),
            name: name.into(),
            description: description.into(),
            coa_id,
            default_interest_rate: behave_policy.get_default_interest_rate(),
            default_min_balance: behave_policy.get_default_min_balance(),
        }
    }
}
