use sqlx::{PgPool, Postgres, Transaction};

use crate::user::models::{UpdateUserEntity, UserEntity};
use crate::user::schemas::UserResponse;

pub struct UserRepository<'a, 'b> {
    pool: &'a PgPool,
    tx: &'b mut Transaction<'a, Postgres>,
}

impl<'a, 'b> UserRepository<'a, 'b> {
    pub fn from(pool: &'a PgPool, tx: &'b mut Transaction<'a, Postgres>) -> Self {
        Self { pool, tx }
    }

    #[tracing::instrument("Saving user details in the database", skip(self, user))]
    pub async fn create(&mut self, user: &UserEntity) -> Result<(), sqlx::Error> {
        sqlx::query(
        "INSERT INTO tuser(id, first_name, last_name, username, password, date_of_birth, email, is_confirmed, is_active, is_verified, access_role)
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(user.id)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.date_of_birth)
        .bind(&user.email)
        .bind(user.is_confirmed)
        .bind(user.is_active)
        .bind(user.is_verified)
        .bind(&user.access_role)
        .execute(&mut **self.tx)
        .await?;

        Ok(())
    }

    pub async fn update(&self, update_user_entity: &UpdateUserEntity) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE tuser SET first_name = COALESCE($1, first_name)
                                            last_name = COALESCE($2, last_name)
                                            username = COALESCE($3, username)
                                            password = COALESCE($4, password)
                                            date_of_birth = COALESCE($5, date_of_birth)
                                            email = COALESCE($6, email)
                                            is_confirmed = COALESCE($7, is_confirmed)
                                            is_active = COALESCE($8, is_active)
                                            is_verified = COALESCE($9, is_verified)
                                            access_role = COALESCE($10, access_role)
                                        WHERE id = $11",
        )
        .bind(update_user_entity.get_first_name())
        .bind(update_user_entity.get_last_name())
        .bind(update_user_entity.get_username())
        .bind(update_user_entity.get_password())
        .bind(update_user_entity.get_date_of_birth())
        .bind(update_user_entity.get_email())
        .bind(update_user_entity.get_is_confirmed())
        .bind(update_user_entity.get_is_active())
        .bind(update_user_entity.get_is_verified())
        .bind(update_user_entity.get_access_role())
        .bind(update_user_entity.get_id())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument("Pulling user details from the database", skip(pool, user_email))]
    pub async fn get_user_by_email(
        pool: &PgPool,
        user_email: &str,
    ) -> Result<Option<UserResponse>, sqlx::Error> {
        let result = sqlx::query_as::<_, UserResponse>("SELECT * FROM tuser WHERE email=$1")
            .bind(user_email)
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    #[tracing::instrument("Updating confirmation in db", skip(pool, user_email))]
    pub async fn confirm_user(pool: &PgPool, user_email: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE tuser SET is_confirmed=true WHERE email=$1")
            .bind(user_email)
            .execute(pool)
            .await?;

        Ok(())
    }
}
