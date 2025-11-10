use crate::account::models::{AccountBalanceEntity, UserAccountEntity};
use crate::ledger::models::LineType;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug)]
pub struct AccountRepository<'a, 'b> {
    pool: &'a PgPool,
    tx: &'b mut Transaction<'a, Postgres>,
}

impl<'a, 'b> AccountRepository<'a, 'b> {
    pub fn from(pool: &'a PgPool, tx: &'b mut Transaction<'a, Postgres>) -> Self {
        Self { pool, tx }
    }

    #[tracing::instrument(
        "Inserting accounts details on creation of user account",
        skip(self, user_account)
    )]
    pub async fn create(&mut self, user_account: &UserAccountEntity) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO user_account(id, user_id, account_number, iban, account_class, coa_id, branch_id, currency, status) 
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(user_account.id)
            .bind(user_account.user_id)
            .bind(&user_account.account_number)
            .bind(&user_account.iban)
            .bind(user_account.account_class)
            .bind(user_account.coa_id)
            .bind(user_account.branch_id)
            .bind(&user_account.currency)
            .bind(&user_account.status)
            .execute(&mut **self.tx).await?;

        Ok(())
    }

    #[tracing::instrument("Fetching chart account id by id", skip(self, account_id))]
    pub async fn fetch_coa_id_by_account_id(
        &self,
        account_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let result: Option<Uuid> = sqlx::query("SELECT coa_id FROM user_account WHERE id=$1")
            .bind(account_id)
            .fetch_optional(self.pool)
            .await?
            .map(|r| r.get("coa_id"));

        Ok(result)
    }

    #[tracing::instrument("Fetching account balance by id", skip(self, account_id))]
    pub async fn fetch_balance_by_user_account_id(
        &self,
        account_id: Uuid,
    ) -> Result<Option<AccountBalanceEntity>, sqlx::Error> {
        let result = sqlx::query_as::<_, AccountBalanceEntity>(
            "SELECT account_id, amount_cents FROM account_balance WHERE account_id=$1",
        )
        .bind(account_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(result)
    }

    #[tracing::instrument("Initialize account balance cache", skip(self, account_id))]
    pub async fn start_acc_balance(&mut self, account_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO account_balance(account_id, amount_cents) VALUES($1, $2)")
            .bind(account_id)
            .bind(0)
            .execute(&mut **self.tx)
            .await?;

        Ok(())
    }

    #[tracing::instrument("Update account balance cache", skip(self, account_id, amount))]
    pub async fn update_acc_balance(
        &mut self,
        account_id: Uuid,
        amount: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE account_balance
                    SET amount_cents WHERE account_id=$1",
        )
        .bind(amount)
        .bind(account_id)
        .execute(&mut **self.tx)
        .await?;

        Ok(())
    }

    #[tracing::instrument("Calculate account balance", skip(self))]
    pub async fn calculate_acc_balance(
        &mut self,
        account_id: Uuid,
        coa_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "SELECT SUM(COALESCE(CASE jl.line_type 
                        WHEN $1 THEN jl.amount_cents
                        WHEN $2  THEN -jl.amount_cents
                        END, 0)) AS balance_cents
                    FROM journal_entry je JOIN journal_line jl ON je.id = jl.journal_entry_id 
                    WHERE je.user_account_id=$3
                    AND jl.coa_id = $3;",
        )
        .bind(LineType::Credit)
        .bind(LineType::Debit)
        .bind(account_id)
        .bind(coa_id)
        .fetch_one(&mut **self.tx)
        .await?
        .get("balance_cents");

        Ok(result)
    }
}
