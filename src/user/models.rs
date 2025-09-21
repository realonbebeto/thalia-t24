use crate::base::error::ValidationError;
use crate::base::{Email, Name, Username};
use crate::user::schemas::UserCreateRequest;
use error_stack::Report;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum AccessRole {
    Superuser,
    Manager,
    Customer,
}

impl From<String> for AccessRole {
    fn from(value: String) -> Self {
        match value.to_lowercase().trim() {
            "superuser" => AccessRole::Superuser,
            "manager" => AccessRole::Manager,
            "customer" => AccessRole::Customer,
            _ => AccessRole::Customer,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub first_name: Name,
    pub last_name: Name,
    pub username: Username,
    pub email: Email,
    pub date_of_birth: chrono::NaiveDate,
    pub is_active: bool,
    pub is_verified: bool,
    pub access_role: AccessRole,
}

impl TryFrom<UserCreateRequest> for User {
    type Error = Report<ValidationError>;

    fn try_from(value: UserCreateRequest) -> Result<Self, Self::Error> {
        let first_name = Name::parse(value.first_name)?;
        let last_name = Name::parse(value.last_name)?;
        let email = Email::parse(value.email)?;
        let username = Username::parse(value.username)?;
        let is_active = false;
        let is_verified = false;
        let access_role: AccessRole = value.access_role.into();
        let date_of_birth = value.date_of_birth;

        Ok(User {
            id: Uuid::now_v7(),
            first_name,
            last_name,
            username,
            email,
            date_of_birth,
            is_active,
            is_verified,
            access_role,
        })
    }
}
