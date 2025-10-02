use crate::telemetry::TraceError;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[tracing::instrument("Saving activate token in the database", skip(tx))]
pub async fn db_store_token(
    tx: &mut Transaction<'_, Postgres>,
    token: &str,
    user_id: Uuid,
    user_email: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO activate_token(token, user_id, user_email) VALUES($1, $2, $3)")
        .bind(token)
        .bind(user_id)
        .bind(user_email)
        .execute(&mut **tx)
        .await
        .trace_with("Error while inserting activate token")?;

    Ok(())
}

#[tracing::instrument("Retrieving password for a username", skip(pool))]
pub async fn db_get_password_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<(Uuid, String, String), sqlx::Error> {
    let result = sqlx::query("SELECT id, username, password FROM tuser WHERE username=$1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .trace_with("Error while fetching password string")?;

    match result {
        Some(r) => Ok((
            r.get::<Uuid, _>("id"),
            r.get::<String, _>("username"),
            r.get::<String, _>("password"),
        )),
        None => Err(sqlx::Error::RowNotFound)
            .trace_with(&format!("Password for username: {} not found", username)),
    }
}

#[tracing::instrument("Retrieving password for an email", skip(pool))]
pub async fn db_get_password_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<(Uuid, String, String), sqlx::Error> {
    let result = sqlx::query("SELECT id, username, password FROM tuser WHERE email=$1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .trace_with("Error while fetching password string")?;

    match result {
        Some(r) => Ok((
            r.get::<Uuid, _>("id"),
            r.get::<String, _>("username"),
            r.get::<String, _>("password"),
        )),
        None => Err(sqlx::Error::RowNotFound)
            .trace_with(&format!("Password for email: {} not found", email)),
    }
}
