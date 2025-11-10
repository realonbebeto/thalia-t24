use chrono::Utc;
use sqlx::types::chrono;
use std::str::FromStr;
use uuid::Uuid;

use crate::authentication::token::ActivateClaims;
use crate::base::error::DomainError;
use crate::base::{Email, Name, Password, Username, error::AppError};
use crate::user::models::{AccessRole, UserEntity};

#[derive(Debug, utoipa::ToSchema, serde::Deserialize)]
pub struct UserRegisterRequest {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: chrono::NaiveDate,
    pub username: String,
    pub password: String,
    pub email: String,
    pub access_role: String,
}

#[derive(sqlx::FromRow)]
#[allow(unused)]
pub struct UserResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub access_role: String,
}

// Domain Type
#[derive(Debug, getset::Getters)]
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

impl User {
    pub fn from_register(register_req: UserRegisterRequest) -> Result<User, AppError> {
        // Check if a user is age > 18
        let age_days = (Utc::now().date_naive() - register_req.date_of_birth).num_days();
        if age_days < 18 * 365 {
            return Err(AppError::Domain(DomainError::ConstraintViolation(
                "You need to be 18 or older to use this service. Please try again when you meet the age requirement.".into(),
            )))?;
        }

        let email = Email::parse(register_req.email)?;

        let password = Password::parse(register_req.password)?;

        let access_role = AccessRole::from_str(&register_req.access_role)?;

        let username = Username::parse(register_req.username)?;

        let first_name = Name::parse(register_req.first_name, "first_name")?;

        let last_name = Name::parse(register_req.last_name, "last_name")?;

        Ok(Self {
            id: Uuid::now_v7(),
            first_name,
            last_name,
            email,
            password,
            username,
            access_role,
            is_confirmed: false,
            is_verified: false,
            is_active: true,
            date_of_birth: register_req.date_of_birth,
        })
    }

    pub fn to_activate_claims(&self, exp: usize) -> ActivateClaims {
        ActivateClaims::from(
            self.email.clone(),
            self.id,
            exp,
            self.access_role.clone(),
            "sign-up".into(),
        )
    }
}

impl TryFrom<User> for UserEntity {
    type Error = anyhow::Error;

    fn try_from(value: User) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            first_name: value.first_name.as_ref().to_string(),
            last_name: value.last_name.as_ref().to_string(),
            username: value.username.as_ref().to_string(),
            password: value.password.encode_password()?,
            email: value.email.to_string(),
            date_of_birth: value.date_of_birth,
            is_confirmed: value.is_confirmed,
            is_active: value.is_active,
            is_verified: value.is_verified,
            access_role: value.access_role,
        })
    }
}
