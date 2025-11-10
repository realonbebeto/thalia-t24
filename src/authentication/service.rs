use actix_web_flash_messages::FlashMessage;

use crate::authentication::{
    SessionType,
    {
        credential::Credentials,
        schemas::{LoginIdentifier, LoginRequest},
        session_handler,
    },
};
use crate::base::error::{AppError, AuthError, SqlErrorExt};
use crate::config::state::AppState;
use crate::infra::pgdb::UnitofWork;
use crate::user::models::UpdateUserEntity;

pub struct AuthService<'a> {
    app_state: &'a AppState,
}

impl<'a> AuthService<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn verify_user_email(&self, token: String) -> Result<(), AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let activate_claims = self
            .app_state
            .activate_handler
            .verify_activate_token(&token)?;

        let update_user_entity =
            UpdateUserEntity::activate_email_update(activate_claims.get_user_id());

        uow.users()
            .update(&update_user_entity)
            .await
            .to_app_err("Failed to update user")?;

        uow.commit()
            .await
            .to_app_err("Failed to commit user email verificaion")?;

        Ok(())
    }

    #[tracing::instrument("Customer login", skip(self, login_req, session))]
    pub async fn authenticate_user<T: SessionType>(
        &self,
        login_req: LoginRequest,
        session: T,
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

        let creds = match credentials
            .validate_credentials(&self.app_state.pgpool)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                FlashMessage::info(format!("{} login unsuccessful", session.kind())).send();
                Err(e)?
            }
        };

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
}
