use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::base::{
    error::{AppError, AuthError},
    {Email, Password},
};
use crate::config::state::DefaultPassword;
use crate::infra::pgdb::UnitofWork;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::user::models::{AccessRole, UserEntity};

#[derive(Debug)]
pub struct ValidCreds {
    pub id: Uuid,
    pub email: Email,
    pub role: AccessRole,
}

impl TryFrom<UserEntity> for ValidCreds {
    type Error = anyhow::Error;
    fn try_from(value: UserEntity) -> Result<Self, Self::Error> {
        let email = Email::parse(value.email)?;
        Ok(Self {
            id: value.id,
            email,
            role: value.access_role,
        })
    }
}

#[derive(Debug, getset::Getters)]
#[get = "pub with_prefix"]
pub struct Credentials {
    email: Email,
    password: Password,
    // default password is passed as env variable used to limit timing attack
    default_password: DefaultPassword,
}

impl Credentials {
    pub fn from(
        email: String,
        password: String,
        default_password: &DefaultPassword,
    ) -> Result<Self, AppError> {
        let email = Email::parse(email)?;
        let password = Password::parse(password)?;

        Ok(Self {
            email,
            password,
            default_password: default_password.clone(),
        })
    }

    pub async fn validate_credentials(&self, pool: &PgPool) -> Result<ValidCreds, anyhow::Error> {
        let mut uow = UnitofWork::from(pool)
            .await
            .context("Failed to start postgres uow")?;

        // Used to limit timing attack
        let mut user_cred: Option<ValidCreds> = None;
        let mut expected_password = self.get_default_password().0.clone();

        if let Some(raw_cred) = uow
            .authentication()
            .fetch_password_by_email(self.get_email().as_ref())
            .await
            .context("Failed to fetch password by email")?
        {
            // Replace password with retrieved password
            expected_password = raw_cred.password.clone();
            user_cred = Some(raw_cred.try_into()?);
        }

        let curr_pass = self.get_password().as_ref().to_owned();

        spawn_blocking_with_tracing(move || {
            Password::verify_password(&expected_password.clone(), &curr_pass)
        })
        .await
        .unwrap()?;

        user_cred.ok_or(anyhow::anyhow!(AuthError::InvalidCredentials(
            "Password or Username".into(),
        )))
    }
}
