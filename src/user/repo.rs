use super::schemas::UserResponse;
use crate::user::models::User;
use sqlx::{PgPool, Postgres, Transaction};

#[tracing::instrument("Saving user details in the database")]
pub async fn db_create_user(
    tx: &mut Transaction<'_, Postgres>,
    user: &User,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tuser(id, first_name, last_name, username, password, date_of_birth, email, is_active, access_role)
        VALUES($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(user.id)
    .bind(user.first_name.as_ref())
    .bind(user.last_name.as_ref())
    .bind(user.username.as_ref())
    .bind(user.password.phash_as_ref())
    .bind(user.date_of_birth)
    .bind(user.email.as_ref())
    .bind(user.is_active)
    .bind(&user.access_role)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[tracing::instrument("Pulling user details from the database")]
pub async fn db_get_user(
    pool: &PgPool,
    user_email: &str,
) -> Result<Option<UserResponse>, sqlx::Error> {
    let result = sqlx::query_as::<_, UserResponse>("SELECT * FROM tuser WHERE email=$1")
        .bind(user_email)
        .fetch_optional(pool)
        .await?;

    Ok(result)
}

#[tracing::instrument("Pulling user details from the database")]
pub async fn db_activate_user(pool: &PgPool, user_email: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tuser SET is_active=true WHERE email=$1")
        .bind(user_email)
        .execute(pool)
        .await?;

    Ok(())
}
