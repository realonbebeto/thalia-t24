use crate::base::error::ValidationError;
use crate::base::{Email, Name, Password, Username};
use crate::user::schemas::UserCreateRequest;
use error_stack::Report;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Type, PartialEq)]
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

#[derive(
    Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, getset::Getters, getset::Setters,
)]
#[get = "pub with_prefix"]
pub struct User {
    pub id: Uuid,
    pub first_name: Name,
    pub last_name: Name,
    pub username: Username,
    pub password: Password,
    pub email: Email,
    pub date_of_birth: chrono::NaiveDate,
    pub is_confirmed: bool,
    pub is_active: bool,
    pub is_verified: bool,
    pub access_role: AccessRole,
}

pub struct StaffUser(pub User);

pub struct CustomerUser(pub User);

impl TryFrom<UserCreateRequest> for StaffUser {
    type Error = Report<ValidationError>;

    fn try_from(value: UserCreateRequest) -> Result<Self, Self::Error> {
        let first_name = Name::parse(value.first_name)?;
        let last_name = Name::parse(value.last_name)?;
        let email = Email::parse(value.email)?;
        let username = Username::parse(value.username)?;
        let password = Password::parse(value.password)?;
        let is_confirmed = false;
        let is_active = false;
        let is_verified = false;
        let date_of_birth = value.date_of_birth;
        let access_role: AccessRole = value.access_role.into();

        if access_role == AccessRole::Customer {
            return Err(Report::new(ValidationError::WrongAccessRole));
        }

        Ok(StaffUser(User {
            id: Uuid::now_v7(),
            first_name,
            last_name,
            username,
            password,
            email,
            date_of_birth,
            is_confirmed,
            is_active,
            is_verified,
            access_role,
        }))
    }
}

impl TryFrom<UserCreateRequest> for CustomerUser {
    type Error = Report<ValidationError>;

    fn try_from(value: UserCreateRequest) -> Result<Self, Self::Error> {
        let first_name = Name::parse(value.first_name)?;
        let last_name = Name::parse(value.last_name)?;
        let email = Email::parse(value.email)?;
        let username = Username::parse(value.username)?;
        let password = Password::parse(value.password)?;
        let is_confirmed = false;
        let is_active = false;
        let is_verified = false;
        let date_of_birth = value.date_of_birth;

        let access_role: AccessRole = value.access_role.into();

        if access_role != AccessRole::Customer {
            return Err(Report::new(ValidationError::WrongAccessRole));
        }

        Ok(CustomerUser(User {
            id: Uuid::now_v7(),
            first_name,
            last_name,
            username,
            password,
            email,
            date_of_birth,
            is_confirmed,
            is_active,
            is_verified,
            access_role,
        }))
    }
}
