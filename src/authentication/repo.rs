use error_stack::{Report, ResultExt};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::base::error::DBError;

#[tracing::instrument("Saving activate token in the database", skip(tx))]
pub async fn db_store_token(
    tx: &mut Transaction<'_, Postgres>,
    token: &str,
    user_id: Uuid,
    user_email: &str,
) -> Result<(), Report<DBError>> {
    sqlx::query("INSERT INTO activate_token(token, user_id, user_email) VALUES($1, $2, $3)")
        .bind(token)
        .bind(user_id)
        .bind(user_email)
        .execute(&mut **tx)
        .await
        .change_context(DBError::DBFault {
            message: "Error while inserting activate token".into(),
        })?;

    Ok(())
}

#[tracing::instrument("Retrieving password for a username", skip(pool))]
pub async fn db_get_password_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<(Uuid, String), Report<DBError>> {
    let result = sqlx::query("SELECT id, password FROM tuser WHERE username=$1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .change_context(DBError::DBFault {
            message: "Error while fetching password string".into(),
        })?;

    match result {
        Some(r) => Ok((r.get::<Uuid, _>("id"), r.get::<String, _>("password"))),
        None => Err(Report::new(DBError::NotFound)
            .attach(format!("Password for username: {} not found", username))),
    }
}

#[tracing::instrument("Retrieving password for an email", skip(pool))]
pub async fn db_get_password_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<(Uuid, String), Report<DBError>> {
    let result = sqlx::query("SELECT id, password FROM tuser WHERE email=$1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .change_context(DBError::DBFault {
            message: "Error while fetching password string".into(),
        })?;

    match result {
        Some(r) => Ok((r.get::<Uuid, _>("id"), r.get::<String, _>("password"))),
        None => Err(Report::new(DBError::NotFound)
            .attach(format!("Password for email: {} not found", email))),
    }
}
