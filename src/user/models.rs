use derive_more::Display;
use sqlx::types::chrono;
use std::str::FromStr;
use uuid::Uuid;

use crate::base::error::ValidationError;

#[derive(
    Debug, serde::Deserialize, serde::Serialize, sqlx::Type, PartialEq, Eq, Clone, Hash, Display,
)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum AccessRole {
    Superuser,
    Manager,
    Customer,
}
impl FromStr for AccessRole {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "superuser" => Ok(AccessRole::Superuser),
            "manager" => Ok(AccessRole::Manager),
            "customer" => Ok(AccessRole::Customer),
            _ => Err(ValidationError::InvalidValue {
                field: "access_role".into(),
                reason: "Unknown access_role".into(),
            }),
        }
    }
}

#[derive(
    Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, getset::Getters, getset::Setters,
)]
#[get = "pub with_prefix"]
pub struct UserEntity {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub date_of_birth: chrono::NaiveDate,
    pub is_confirmed: bool,
    pub is_active: bool,
    pub is_verified: bool,
    pub access_role: AccessRole,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow, getset::Getters)]
#[get = "pub with_prefix"]
pub struct UpdateUserEntity {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub is_confirmed: Option<bool>,
    pub is_active: Option<bool>,
    pub is_verified: Option<bool>,
    pub access_role: Option<AccessRole>,
}

impl UpdateUserEntity {
    pub fn activate_email_update(user_id: Uuid) -> Self {
        Self {
            id: user_id,
            first_name: None,
            last_name: None,
            username: None,
            password: None,
            email: None,
            date_of_birth: None,
            is_confirmed: Some(true),
            is_active: None,
            is_verified: None,
            access_role: None,
        }
    }
}
