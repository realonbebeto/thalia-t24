use std::str::FromStr;
use uuid::Uuid;

use crate::base::error::{AppError, AuthError, DomainError, SqlErrorExt};
use crate::config::state::AppState;
use crate::infra::pgdb::UnitofWork;
use crate::staff::{
    models::{ChartAccount, CoaType, CustomerAccountType},
    schemas::{AccountTypeRequest, ChartAccountRequest},
};

use crate::authentication::{
    StaffSession,
    credential::Credentials,
    schemas::{LoginIdentifier, LoginRequest},
    session_handler,
};

pub struct StaffService<'a> {
    app_state: &'a AppState,
}

impl<'a> StaffService<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    #[tracing::instrument("Staff Login", skip(self, login_req, session))]
    pub async fn authenticate_staff(
        &self,
        login_req: LoginRequest,
        session: StaffSession,
    ) -> Result<(String, String), AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let email = match login_req.login_id {
            LoginIdentifier::Email(v) => v,
            LoginIdentifier::Username(username) => {
                match uow
                    .authentication()
                    .fetch_password_by_username(&username)
                    .await
                    .to_app_err("Failed to fetch user entity")?
                {
                    Some(u) => u.email,
                    None => Err(AuthError::InvalidCredentials("Password or Username".into()))?,
                }
            }
        };

        let credentials =
            Credentials::from(email, login_req.password, &self.app_state.default_password)?;

        let creds = credentials
            .validate_credentials(&self.app_state.pgpool)
            .await?;

        let (pair, _) = session_handler(
            &self.app_state.token_handler,
            &self.app_state.redis_pool,
            &session,
            creds.id,
            creds.email,
            creds.role,
        )
        .await?;

        Ok((pair.access_token, pair.refresh_token))
    }

    #[tracing::instrument("Create account type", skip(self))]
    pub async fn account_type_creation(&self, request: AccountTypeRequest) -> Result<(), AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let acc_type = CustomerAccountType {
            id: Uuid::now_v7(),
            coa_id: request.coa_id,
            description: request.description,
            name: request.name,
        };

        uow.staffs()
            .create_account_type(&acc_type)
            .await
            .to_app_err("Failed to create account type")?;

        Ok(())
    }

    #[tracing::instrument("Create chart account", skip(self))]
    pub async fn chart_account_creation(
        &self,
        request: ChartAccountRequest,
    ) -> Result<(), AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let coa_type = CoaType::from_str(&request.coa_type)?;

        let coa = ChartAccount::new(
            Uuid::now_v7(),
            &request.code,
            &request.name,
            coa_type,
            &request.currency,
        );

        if uow
            .staffs()
            .fetch_coa_by_code(&request.code)
            .await
            .to_app_err("Failed to fetch coa by code")?
            .is_some()
        {
            Err(DomainError::Duplicate("coa already exists".into()))?
        }

        uow.staffs()
            .create_chart_account(&coa)
            .await
            .to_app_err("Failed to create chart account")?;

        Ok(())
    }
}
