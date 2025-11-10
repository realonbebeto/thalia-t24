use crate::{
    base::error::{AppError, SqlErrorExt},
    config::state::AppState,
    infra::pgdb::UnitofWork,
    user::{
        models::UserEntity,
        schemas::{User, UserRegisterRequest},
    },
};

pub struct UserService<'a> {
    app_state: &'a AppState,
}

impl<'a> UserService<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    #[tracing::instrument("Create user account", skip(self))]
    pub async fn create_user(&self, user_req: UserRegisterRequest) -> Result<(), AppError> {
        let mut uow = UnitofWork::from(&self.app_state.pgpool)
            .await
            .to_app_err("Failed to start postgres uow")?;

        let user = User::from_register(user_req)?;

        let activate_token = self
            .app_state
            .activate_handler
            .generate_activate_token(&user, &self.app_state.redis_pool)
            .await?;

        let user_entity: UserEntity = user.try_into()?;

        uow.users()
            .create(&user_entity)
            .await
            .to_app_err(&format!("Failed to create {}", user_entity.access_role))?;

        uow.authentication()
            .store_token(
                &activate_token,
                *user_entity.get_id(),
                user_entity.get_email(),
            )
            .await
            .to_app_err("Failed to store activate token")?;

        // Send Activate Email
        self.app_state
            .email_client
            .send_welcome_email(
                &self.app_state.base_uri.0,
                &user_entity.email,
                "Welcome to Thalia Corp.",
                user_entity.first_name.as_ref(),
                &activate_token,
                "Thalia Corp.",
            )
            .await?;

        uow.commit().await.to_app_err(&format!(
            "Failed to commit {} creation",
            user_entity.access_role
        ))?;

        Ok(())
    }
}
