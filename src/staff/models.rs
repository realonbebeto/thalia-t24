use std::str::FromStr;

use strum::Display;
use uuid::Uuid;

use crate::base::error::ValidationError;

#[derive(Debug, sqlx::Type, Display)]
#[sqlx(type_name = "chart_account_type", rename_all = "lowercase")]
pub enum CoaType {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
    Memoranda,
}

impl FromStr for CoaType {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "asset" => Ok(CoaType::Asset),
            "equity" => Ok(CoaType::Equity),
            "expense" => Ok(CoaType::Expense),
            "income" => Ok(CoaType::Income),
            "liability" => Ok(CoaType::Liability),
            "memoranda" => Ok(CoaType::Memoranda),
            _ => Err(ValidationError::InvalidValue {
                field: "coa_type".into(),
                reason: "Unknown coa_type".into(),
            }),
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
    pub fn new(id: Uuid, code: &str, name: &str, coa_type: CoaType, currency: &str) -> Self {
        Self {
            id,
            code: code.into(),
            name: name.into(),
            coa_type,
            currency: currency.into(),
        }
    }
}

pub struct CustomerAccountType {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub coa_id: Uuid,
}
