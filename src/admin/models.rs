use error_stack::Report;
use strum::Display;
use uuid::Uuid;

#[derive(Debug, sqlx::Type, Display)]
#[sqlx(type_name = "chart_account_type", rename_all = "lowercase")]
pub enum CoaType {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

#[derive(Debug, thiserror::Error)]
pub enum CoaTypeError {
    #[error("Invalid chart account type: {message}")]
    Invalid { message: String },
}

impl TryFrom<String> for CoaType {
    type Error = Report<CoaTypeError>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().trim() {
            "asset" => Ok(CoaType::Asset),
            "equity" => Ok(CoaType::Equity),
            "expense" => Ok(CoaType::Expense),
            "income" => Ok(CoaType::Income),
            "liability" => Ok(CoaType::Liability),
            _ => Err(Report::new(CoaTypeError::Invalid { message: value })),
        }
    }
}

#[derive(Debug, sqlx::FromRow, getset::Getters)]
#[get = "pub with_prefix"]
pub struct ChartAccount {
    id: Uuid,
    code: String,
    name: String,
    coa_type: CoaType,
    currency: String,
}

impl ChartAccount {
    pub fn new(id: Uuid, code: &str, name: String, coa_type: CoaType, currency: String) -> Self {
        Self {
            id,
            code: code.into(),
            name,
            coa_type,
            currency,
        }
    }
}

pub struct CustomerAccountType {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub coa_id: Uuid,
}
