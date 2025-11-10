use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::user::models::UserEntity;

pub struct AuthRepository<'a, 'b> {
    pool: &'a PgPool,
    tx: &'b mut Transaction<'a, Postgres>,
}

impl<'a, 'b> AuthRepository<'a, 'b> {
    pub fn from(pool: &'a PgPool, tx: &'b mut Transaction<'a, Postgres>) -> Self {
        Self { pool, tx }
    }

    #[tracing::instrument("Saving activate token in the database", skip(self))]
    pub async fn store_token(
        &mut self,
        token: &str,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO activate_token(token, user_id, user_email) VALUES($1, $2, $3)")
            .bind(token)
            .bind(user_id)
            .bind(user_email)
            .execute(&mut **self.tx)
            .await?;

        Ok(())
    }

    #[tracing::instrument("Retrieving password for a username", skip(self))]
    pub async fn fetch_password_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let result = sqlx::query_as::<_, UserEntity>(
            "SELECT id, first_name, last_name, username, password, email, date_of_birth, is_confirmed, is_active, is_verified, access_role 
                FROM tuser WHERE username=$1",
        )
        .bind(username)
        .fetch_optional(self.pool)
        .await?;

        Ok(result)
    }

    #[tracing::instrument("Retrieving password for an email", skip(self))]
    pub async fn fetch_password_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let result = sqlx::query_as::<_, UserEntity>("SELECT id, first_name, last_name, username, password, email, date_of_birth, is_confirmed, is_active, is_verified, access_role  FROM tuser WHERE email=$1")
            .bind(email)
            .fetch_optional(self.pool)
            .await?;

        Ok(result)
    }
}
