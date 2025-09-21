use error_stack::{Report, ResultExt};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::base::error::DBError;

#[tracing::instrument("Saving activate token in the database")]
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
